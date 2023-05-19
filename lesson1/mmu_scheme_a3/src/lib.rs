#![no_std]
#![feature(asm_const)]


use riscv::register::satp;

pub const KERNEL_BASE: usize = 0xffff_ffff_c000_0000;
const PHYS_VIRT_OFFSET: usize = 0xffff_ffc0_0000_0000;
const MMU_LEVELS: usize = if cfg!(feature = "sv39") {
    3
} else if cfg!(feature = "sv48") {
    4
} else if cfg!(feature = "sv57") {
    5
} else { 0 };
struct PageTable([usize; 512]);
#[link_section = ".data.boot_page_table"]
static mut PageTables:[PageTable; 4] = [PageTable([0; 512]), PageTable([0; 512]), PageTable([0; 512]), PageTable([0; 512])];
static mut allocated_id: usize = 1; // 0 is root_page_table
fn alloc_page() -> *mut PageTable {
    unsafe {
        allocated_id += 1;
        &mut PageTables[allocated_id - 1]
    }
}
fn boot_map<F1/*, F2*/>(table: &mut PageTable, level: usize, va: usize, pa: usize,
    len: usize, prot: usize, alloc_page: &mut F1/*, phys_to_virt: &F2*/)
    -> Result<(), isize> where F1: FnMut() -> *mut PageTable/*, F2: Fn(PAddr) -> *mut PageTable*/
{
    let pte: &mut usize = &mut table.0[((va >> 12) >> ((level - 1) * 9)) & 0x1FF];
    if (*pte & 0x1) == 0 {
        // 页表项是缺失的
        *pte = if level > 3 { (alloc_page() as usize >> 12 << 10) | 0xef } else { (pa >> 12 << 10) | 0x01 };
    }
    if level > 3 {
        unsafe {
            return boot_map(&mut *((*pte >> 10 << 12) as *mut PageTable), level - 1, va, pa, len, prot, alloc_page/*, phys_to_virt*/);
        }
    } else {
        return Ok(());
    }
}
pub unsafe fn pre_mmu() {
    if !cfg!(feature = "disable") {
        boot_map(&mut PageTables[0], MMU_LEVELS, 0x8000_0000, 0x8000_0000, 0, 0, &mut alloc_page);
        boot_map(&mut PageTables[0], MMU_LEVELS, 0xffff_ffc0_8000_0000, 0x8000_0000, 0, 0, &mut alloc_page);
        boot_map(&mut PageTables[0], MMU_LEVELS, 0xffff_ffff_c000_0000, 0x8000_0000, 0, 0, &mut alloc_page);
    }
    // 0x8000_0000..0xc000_0000, VRWX_GAD, 1G block
    //BOOT_PT_SV39[2] = (0x80000 << 10) | 0xef; // 似乎是跳板？
    // 只有 R W X 全为 0 时才不是叶子节点，此时已经是叶子了。
    // 0xffff_ffc0_8000_0000..0xffff_ffc0_c000_0000, VRWX_GAD, 1G block // 左闭右开区间
    //BOOT_PT_SV39[0x102] = (0x80000 << 10) | 0xef; // 链接时把内核代码段放这里了

    // 0xffff_ffff_c000_0000..highest, VRWX_GAD, 1G block
    //BOOT_PT_SV39[0x1ff] = (0x80000 << 10) | 0xef; // e3 is ok // 从 KERNEL_BASE 开始
    // ffff_ffff_c000_0000 -> 0000_0000_8000_0000 内核在物理地址 0x80200000
}
pub unsafe fn enable_mmu() {
    if cfg!(feature = "sv39") {
        let page_table_root = PageTables.as_ptr() as usize;
        satp::set(satp::Mode::Sv39, 0, page_table_root >> 12);
        riscv::asm::sfence_vma_all();
        
    } else if cfg!(feature = "sv48") {
        let page_table_root = PageTables.as_ptr() as usize;
        satp::set(satp::Mode::Sv48, 0, page_table_root >> 12);
        riscv::asm::sfence_vma_all();
    } else if cfg!(feature = "disable") {
        satp::set(satp::Mode::Bare, 0, 0);
        riscv::asm::sfence_vma_all();
    }
}
pub unsafe fn post_mmu() {
    if !cfg!(feature = "disable") {
        // 开启页表后跳转至高地址来访问内核，虽然低地址仍然也是有效的
        core::arch::asm!("
            li      t0, {phys_virt_offset}  // fix up virtual high address
            add     sp, sp, t0
            add     ra, ra, t0
            ret     ",
            phys_virt_offset = const PHYS_VIRT_OFFSET,
        );
    }
}
