#![no_std]
#![feature(asm_const)]

use riscv::register::satp;

pub const KERNEL_BASE: usize = 0xffff_ffff_c000_0000;
const PHYS_VIRT_OFFSET: usize = 0xffff_ffc0_0000_0000;

#[link_section = ".data.boot_page_table"]
static mut BOOT_PT_SV39: [u64; 512] = [0; 512];
pub unsafe fn pre_mmu() {
    // 0x8000_0000..0xc000_0000, VRWX_GAD, 1G block
    BOOT_PT_SV39[2] = (0x80000 << 10) | 0xef; // 似乎是跳板？
    // 只有 R W X 全为 0 时才不是叶子节点，此时已经是叶子了。
    // 0xffff_ffc0_8000_0000..0xffff_ffc0_c000_0000, VRWX_GAD, 1G block // 左闭右开区间
    BOOT_PT_SV39[0x102] = (0x80000 << 10) | 0xef; // 链接时把内核代码段放这里了

    // 0xffff_ffff_c000_0000..highest, VRWX_GAD, 1G block
    BOOT_PT_SV39[0x1ff] = (0x80000 << 10) | 0xef; // e3 is ok // 从 KERNEL_BASE 开始
    // ffff_ffff_c000_0000 -> 0000_0000_8000_0000 内核在物理地址 0x80200000
}
pub unsafe fn enable_mmu() {
    if cfg!(feature = "enable") {
        let page_table_root = BOOT_PT_SV39.as_ptr() as usize;
        satp::set(satp::Mode::Sv39, 0, page_table_root >> 12);
        riscv::asm::sfence_vma_all();
        
    } else if cfg!(feature = "disable") {
        satp::set(satp::Mode::Bare, 0, 0);
        riscv::asm::sfence_vma_all();
    }
}
pub unsafe fn post_mmu() {
    if cfg!(feature = "enable") {
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
