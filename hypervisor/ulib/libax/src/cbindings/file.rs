use super::fd_table::{add_new_file, Filelike};
use super::{ctypes, utils::char_ptr_to_str};
use crate::debug;
use crate::fs::{File, OpenOptions};
use crate::io::{self, prelude::*, SeekFrom};

use axerrno::{LinuxError, LinuxResult};
use core::ffi::{c_char, c_int};

/// Convert open flags to [`OpenOptions`].
fn flags_to_options(flags: c_int, _mode: ctypes::mode_t) -> OpenOptions {
    let flags = flags as u32;
    let mut options = OpenOptions::new();
    match flags & 0b11 {
        ctypes::O_RDONLY => options.read(true),
        ctypes::O_WRONLY => options.write(true),
        _ => options.read(true).write(true),
    };
    if flags & ctypes::O_APPEND != 0 {
        options.append(true);
    }
    if flags & ctypes::O_TRUNC != 0 {
        options.truncate(true);
    }
    if flags & ctypes::O_CREAT != 0 {
        options.create(true);
    }
    if flags & ctypes::O_EXEC != 0 {
        options.create_new(true);
    }
    options
}

/// Open a file by `filename` and insert it into the file descriptor table.
///
/// Return its index in the file table (`fd`). Return `ENFILE` if the file
/// table overflows.
#[no_mangle]
pub unsafe extern "C" fn ax_open(
    filename: *const c_char,
    flags: c_int,
    mode: ctypes::mode_t,
) -> c_int {
    let filename = char_ptr_to_str(filename);
    debug!("ax_open <= {:?} {:#o} {:#o}", filename, flags, mode);
    ax_call_body!(ax_open, {
        let options = flags_to_options(flags, mode);
        let file = options.open(filename?)?;
        add_new_file(file).ok_or(LinuxError::ENFILE)
    })
}

/// Set the position of the file indicated by `fd`.
///
/// Return its position after seek.
#[no_mangle]
pub unsafe extern "C" fn ax_lseek(
    fd: c_int,
    offset: ctypes::off_t,
    whence: c_int,
) -> ctypes::off_t {
    debug!("ax_lseek <= {} {} {}", fd, offset, whence);
    ax_call_body!(ax_lseek, {
        let pos = match whence {
            0 => SeekFrom::Start(offset as _),
            1 => SeekFrom::Current(offset as _),
            2 => SeekFrom::End(offset as _),
            _ => return Err(LinuxError::EINVAL),
        };
        let off = Filelike::from_fd(fd)?.into_file()?.lock().seek(pos)?;
        Ok(off)
    })
}

pub(super) fn stat_file(file: &File) -> io::Result<ctypes::stat> {
    let metadata = file.metadata()?;
    let metadata = metadata.raw_metadata();
    let ty = metadata.file_type() as u8;
    let perm = metadata.perm().bits() as u32;
    let st_mode = ((ty as u32) << 12) | perm;
    Ok(ctypes::stat {
        st_ino: 1,
        st_nlink: 1,
        st_mode,
        st_uid: 1000,
        st_gid: 1000,
        st_size: metadata.size() as _,
        st_blocks: metadata.blocks() as _,
        st_blksize: 512,
        ..Default::default()
    })
}

/// Get the file metadata by `path` and write into `buf`.
///
/// Return 0 if success.
#[no_mangle]
pub unsafe extern "C" fn ax_stat(path: *const c_char, buf: *mut ctypes::stat) -> ctypes::ssize_t {
    let path = char_ptr_to_str(path);
    debug!("ax_stat <= {:?} {:#x}", path, buf as usize);
    ax_call_body!(ax_stat, {
        if buf.is_null() {
            return Err(LinuxError::EFAULT);
        }
        let file = File::open(path?)?;
        let st = stat_file(&file)?;
        drop(file);
        unsafe { *buf = st };
        Ok(0)
    })
}

/// Get the metadata of the symbolic link and write into `buf`.
///
/// Return 0 if success.
#[no_mangle]
pub unsafe extern "C" fn ax_lstat(path: *const c_char, buf: *mut ctypes::stat) -> ctypes::ssize_t {
    let path = char_ptr_to_str(path);
    debug!("ax_lstat <= {:?} {:#x}", path, buf as usize);
    ax_call_body!(ax_lstat, {
        if buf.is_null() {
            return Err(LinuxError::EFAULT);
        }
        unsafe { *buf = Default::default() }; // TODO
        Ok(0)
    })
}

/// Get the path of the current directory.
#[no_mangle]
pub unsafe extern "C" fn ax_getcwd(buf: *mut c_char, size: usize) -> *mut c_char {
    debug!("ax_getcwd <= {:#x} {}", buf as usize, size);
    ax_call_body!(ax_getcwd, {
        if buf.is_null() {
            return Ok(core::ptr::null::<c_char>() as _);
        }
        let dst = unsafe { core::slice::from_raw_parts_mut(buf as *mut u8, size as _) };
        let cwd = crate::env::current_dir()?;
        let cwd = cwd.as_bytes();
        if cwd.len() < size {
            dst[..cwd.len()].copy_from_slice(cwd);
            dst[cwd.len()] = 0;
            Ok(buf)
        } else {
            Err(LinuxError::ERANGE)
        }
    })
}
