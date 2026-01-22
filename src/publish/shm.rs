//! Safe wrapper over LibC shared memory functionality.

#![cfg(feature = "publish_shm")]

use libc::{
    self, O_CREAT, O_RDWR, O_TRUNC, S_IRGRP, S_IROTH, S_IRUSR, S_IWGRP, S_IWUSR, close, ftruncate,
    munmap, pthread_mutex_destroy, pthread_mutex_init, pthread_mutex_lock, pthread_mutex_t,
    pthread_mutex_unlock, shm_open,
};
use std::{io, path::Path, ptr};

/// Opaque type holding the pointer to both a mutex for syncronization and the data we are concerned
/// with sharing with other processes.
pub struct SharedMemory<T>
where
    T: Clone,
{
    /// Some info on the creation of this struct.
    kind: ShmKind,

    /// File descriptor of the shared memory.
    fd: libc::c_int,

    /// POSIX mutex for syncronization.
    mutex: *mut pthread_mutex_t,

    /// The actual shared data, this pointer lives in the same allocation as `mutex`, and should be
    /// a constant offset from it.
    data: *mut T,
}

/// Information about the creation of a [`SharedMemory`] instance, which is useful while performing
/// cleanup, likely via [`Drop`].
///
/// [`SharedMemory`]: SharedMemory
/// [`Drop`]: Drop
#[derive(Debug, Eq, PartialEq)]
enum ShmKind {
    /// Opened, not created.
    Opened,

    /// Created, we're responsible for some extra cleanup work.
    Created,
}

impl<T> SharedMemory<T>
where
    T: Clone,
{
    /// Net size of the shared memory allocation such that it may contain our data and a mutex.
    const SHARED_MEMORY_SIZE: usize = size_of::<pthread_mutex_t>() + size_of::<T>();

    /// Create a new allocation of shared memory for a value of `T`, and initialize it with the
    /// given value of `T`.
    pub fn create(name: impl AsRef<Path>) -> io::Result<SharedMemory<T>> {
        let oflags = O_CREAT | O_RDWR | O_TRUNC;
        let mode = S_IRUSR | S_IWUSR | S_IRGRP | S_IWGRP | S_IROTH;
        let name_bytes = name.as_ref().as_os_str().as_encoded_bytes();
        let mut name_bytes = name_bytes.to_vec();
        name_bytes.push(0);

        unsafe {
            // Open our shared memory as a file descriptor.
            #[cfg(target_os = "macos")]
            let c_str = name_bytes.as_ptr() as *const libc::c_char;
            #[cfg(target_os = "linux")]
            let c_str = name_bytes.as_ptr() as *const u8;

            #[cfg(target_os = "macos")]
            let fd = shm_open(c_str, oflags, [mode]);
            #[cfg(target_os = "linux")]
            let fd = shm_open(c_str, oflags, mode);

            if fd < 0 {
                return Err(io::Error::last_os_error());
            }

            // Truncate the "file" to the correct size.
            if ftruncate(fd, Self::SHARED_MEMORY_SIZE as i64) < 0 {
                return Err(io::Error::last_os_error());
            }

            // Map the "file" to a new memory address.
            let allocation = libc::mmap(
                ptr::null_mut(),
                Self::SHARED_MEMORY_SIZE,
                libc::PROT_WRITE | libc::PROT_READ,
                libc::MAP_SHARED,
                fd,
                0,
            );

            if allocation == libc::MAP_FAILED {
                return Err(io::Error::last_os_error());
            }

            // Divide the memory into a mutex and actual data, then initialize the mutex and data.
            let mutex = allocation as *mut pthread_mutex_t;
            let data = allocation.byte_offset(size_of::<pthread_mutex_t>() as isize) as *mut T;

            match pthread_mutex_init(mutex, ptr::null()) {
                0 => (),
                e => {
                    return Err(io::Error::from_raw_os_error(e));
                }
            };

            Ok(SharedMemory {
                kind: ShmKind::Created,
                fd,
                mutex,
                data,
            })
        }
    }

    /// Open _but do not create_ a [`SharedMemory`] pointing to `T`.
    ///
    /// [`SharedMemory`]: SharedMemory
    pub fn open(name: impl AsRef<Path>) -> io::Result<SharedMemory<T>> {
        let name_bytes = name.as_ref().as_os_str().as_encoded_bytes();
        let mut name_bytes = name_bytes.to_vec();
        name_bytes.push(0);

        // For some notes on the inner workings here see `SharedMemory::create`.

        unsafe {
            #[cfg(target_os = "macos")]
            let c_str = name_bytes.as_ptr() as *const libc::c_char;
            #[cfg(target_os = "linux")]
            let c_str = name_bytes.as_ptr() as *const u8;

            #[cfg(target_os = "macos")]
            let fd = shm_open(c_str, O_RDWR);
            #[cfg(target_os = "linux")]
            let fd = shm_open(c_str, O_RDWR, 0);

            if fd < 0 {
                return Err(io::Error::last_os_error());
            }

            let allocation = libc::mmap(
                ptr::null_mut(),
                Self::SHARED_MEMORY_SIZE,
                libc::PROT_WRITE | libc::PROT_READ,
                libc::MAP_SHARED,
                fd,
                0,
            );

            if allocation == libc::MAP_FAILED {
                return Err(io::Error::last_os_error());
            }

            let mutex = allocation as *mut pthread_mutex_t;
            let data = allocation.byte_offset(size_of::<pthread_mutex_t>() as isize) as *mut T;

            Ok(SharedMemory {
                kind: ShmKind::Opened,
                fd,
                mutex,
                data,
            })
        }
    }

    /// Get the current value pointed to by the given [`SharedMemory`] and return it. This function
    /// requires an exclusive reference to the [`SharedMemory`] as it must internally mutate a
    /// mutex, if multiple references are required you should initialize multiple [`SharedMemory`]
    /// instances.
    ///
    /// [`SharedMemory`]: SharedMemory
    pub fn get(&mut self) -> io::Result<T> {
        unsafe {
            match pthread_mutex_lock(self.mutex) {
                0 => (),
                e => {
                    return Err(io::Error::from_raw_os_error(e));
                }
            };

            let value = (*self.data).clone();

            match pthread_mutex_unlock(self.mutex) {
                0 => (),
                e => {
                    return Err(io::Error::from_raw_os_error(e));
                }
            };

            Ok(value)
        }
    }

    /// Set the value pointed to by the given [`SharedMemory`] to the given value of `T`.
    ///
    /// [`SharedMemory`]: SharedMemory
    pub fn set(&mut self, value: T) -> io::Result<()> {
        unsafe {
            match pthread_mutex_lock(self.mutex) {
                0 => (),
                e => {
                    return Err(io::Error::from_raw_os_error(e));
                }
            };

            *self.data = value;

            match pthread_mutex_unlock(self.mutex) {
                0 => (),
                e => {
                    return Err(io::Error::from_raw_os_error(e));
                }
            };
        }

        Ok(())
    }
}

impl<T> Drop for SharedMemory<T>
where
    T: Clone,
{
    fn drop(&mut self) {
        unsafe {
            if self.kind == ShmKind::Created {
                pthread_mutex_destroy(self.mutex);
            }

            close(self.fd);
            munmap(self.mutex as _, Self::SHARED_MEMORY_SIZE);
        }
    }
}
