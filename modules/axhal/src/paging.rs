//! Page table manipulation.

use axalloc::global_allocator;
use page_table::PagingIf;

use crate::mem::{phys_to_virt, virt_to_phys, MemRegionFlags, PhysAddr, VirtAddr, PAGE_SIZE_4K};

#[doc(no_inline)]
pub use page_table::{MappingFlags, PagingError, PagingResult};

impl From<MemRegionFlags> for MappingFlags {
    fn from(f: MemRegionFlags) -> Self {
        let mut ret = Self::empty();
        if f.contains(MemRegionFlags::READ) {
            ret |= Self::READ;
        }
        if f.contains(MemRegionFlags::WRITE) {
            ret |= Self::WRITE;
        }
        if f.contains(MemRegionFlags::EXECUTE) {
            ret |= Self::EXECUTE;
        }
        if f.contains(MemRegionFlags::DEVICE) {
            ret |= Self::DEVICE;
        }
        if f.contains(MemRegionFlags::UNCACHED) {
            ret |= Self::UNCACHED;
        }
        ret
    }
}

/// Implementation of [`PagingIf`], to provide physical memory manipulation to
/// the [page_table] crate.
pub struct PagingIfImpl;

impl PagingIf for PagingIfImpl {
    fn alloc_frame() -> Option<PhysAddr> {
        global_allocator()
            .alloc_pages(1, PAGE_SIZE_4K)
            .map(|vaddr| virt_to_phys(vaddr.into()))
            .ok()
    }

    fn dealloc_frame(paddr: PhysAddr) {
        global_allocator().dealloc_pages(phys_to_virt(paddr).as_usize(), 1)
    }

    #[inline]
    fn phys_to_virt(paddr: PhysAddr) -> VirtAddr {
        phys_to_virt(paddr)
    }
}

#[cfg(feature = "nrpt")]
mod nr {
    extern crate alloc;

    use super::*;
    use alloc::boxed::Box;
    use nr_page_table::{NrPageTable, NrPteFlags};
    use page_table::PageSize;

    fn mapping_to_nr_flags(flags: MappingFlags) -> NrPteFlags {
        NrPteFlags {
            is_writable: flags.contains(MappingFlags::WRITE),
            is_supervisor: !flags.contains(MappingFlags::USER),
            disable_execute: !flags.contains(MappingFlags::EXECUTE),
            disable_cache: flags.contains(MappingFlags::UNCACHED),
        }
    }

    pub struct PageTable(NrPageTable);

    impl PageTable {
        pub fn try_new() -> PagingResult<Self> {
            let alloc = Box::new(|| {
                let vaddr = global_allocator().alloc_pages(1, PAGE_SIZE_4K).unwrap();
                virt_to_phys(vaddr.into()).into()
            });
            let dealloc = Box::new(|pos| global_allocator().dealloc_pages(pos, 1));
            Ok(Self(NrPageTable::new(
                axconfig::PHYS_VIRT_OFFSET,
                alloc,
                dealloc,
            )))
        }

        pub fn root_paddr(&self) -> PhysAddr {
            self.0.root_paddr().into()
        }

        pub fn map_region(
            &mut self,
            vaddr: VirtAddr,
            paddr: PhysAddr,
            size: usize,
            flags: MappingFlags,
            _allow_huge: bool,
        ) -> PagingResult {
            if !vaddr.is_aligned(PageSize::Size4K.into())
                || !paddr.is_aligned(PageSize::Size4K.into())
                || !memory_addr::is_aligned(size, PageSize::Size4K.into())
            {
                return Err(PagingError::NotAligned);
            }
            trace!(
                "map_region({:#x}): [{:#x}, {:#x}) -> [{:#x}, {:#x}) {:?}",
                self.root_paddr(),
                vaddr,
                vaddr + size,
                paddr,
                paddr + size,
                flags,
            );
            let vaddr = vaddr.as_usize();
            let paddr = paddr.as_usize();
            let flags = mapping_to_nr_flags(flags);
            for i in (0..size).step_by(PAGE_SIZE_4K) {
                self.0
                    .map(vaddr + i, paddr + i, flags)
                    .map_err(|_| PagingError::NrError)?;
            }
            Ok(())
        }
    }
}

cfg_if::cfg_if! {
    if #[cfg(all(feature = "nrpt", target_arch = "x86_64"))] {
        pub type PageTable = nr::PageTable;
    } else if #[cfg(target_arch = "x86_64")] {
        /// The architecture-specific page table.
        pub type PageTable = page_table::x86_64::X64PageTable<PagingIfImpl>;
    } else if #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))] {
        /// The architecture-specific page table.
        pub type PageTable = page_table::riscv::Sv39PageTable<PagingIfImpl>;
    } else if #[cfg(target_arch = "aarch64")]{
        /// The architecture-specific page table.
        pub type PageTable = page_table::aarch64::A64PageTable<PagingIfImpl>;
    }
}
