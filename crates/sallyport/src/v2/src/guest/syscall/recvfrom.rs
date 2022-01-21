// SPDX-License-Identifier: Apache-2.0

use super::types::{Argv, CommittedSockaddrOutput, SockaddrOutput, StagedSockaddrOutput};
use crate::guest::alloc::{Allocator, Collect, Collector, Output, Stage, Syscall};
use crate::Result;

use libc::{c_int, c_long, size_t};

pub struct Recvfrom<'a> {
    pub sockfd: c_int,
    pub buf: &'a mut [u8],
    pub flags: c_int,
    pub src_addr: SockaddrOutput<'a>,
}

unsafe impl<'a> Syscall<'a> for Recvfrom<'a> {
    const NUM: c_long = libc::SYS_recvfrom;

    type Argv = Argv<6>;
    type Ret = size_t;

    type Staged = (
        Output<'a, [u8], &'a mut [u8]>, // buf
        StagedSockaddrOutput<'a>,
    );
    type Committed = (
        Output<'a, [u8], &'a mut [u8]>, // buf
        CommittedSockaddrOutput<'a>,
    );
    type Collected = Option<Result<size_t>>;

    fn stage(self, alloc: &mut impl Allocator) -> Result<(Self::Argv, Self::Staged)> {
        let src_addr = self.src_addr.stage(alloc)?;
        let (buf, _) = Output::stage_slice_max(alloc, self.buf)?;
        Ok((
            Argv([
                self.sockfd as _,
                buf.offset(),
                buf.len(),
                self.flags as _,
                src_addr.addr.offset(),
                src_addr.addrlen.offset(),
            ]),
            (buf, src_addr),
        ))
    }

    fn collect(
        (buf, src_addr): Self::Committed,
        ret: Result<Self::Ret>,
        col: &impl Collector,
    ) -> Self::Collected {
        match ret {
            Ok(ret) if ret > buf.len() => None,
            res @ Ok(ret) => {
                unsafe { buf.collect_range(col, 0..ret) };
                src_addr.collect(col);
                Some(res)
            }
            err => Some(err),
        }
    }
}
