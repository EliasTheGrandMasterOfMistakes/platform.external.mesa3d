/*
 * Copyright © 2022 Collabora, Ltd.
 * SPDX-License-Identifier: MIT
 */

use crate::bitset::*;
use crate::nak_ir::*;

use std::ops::Range;

trait SrcMod {
    fn src_mod(&self, idx: usize) -> u8;
}

impl SrcMod for Instr {
    fn src_mod(&self, idx: usize) -> u8 {
        0 /* TODO */
    }
}

fn encode_imm(bs: &mut impl BitSetMut, range: Range<usize>, imm: &Immediate) {
    assert!(range.len() == 32);
    bs.set_field(range, imm.u);
}

fn encode_reg(bs: &mut impl BitSetMut, range: Range<usize>, reg: RegRef) {
    assert!(range.len() == 8);
    assert!(reg.file() == RegFile::GPR);
    bs.set_field(range, reg.base_idx());
}

fn encode_ureg(bs: &mut impl BitSetMut, range: Range<usize>, reg: RegRef) {
    assert!(range.len() == 8);
    assert!(reg.file() == RegFile::UGPR);
    assert!(reg.base_idx() <= 63);
    bs.set_field(range, reg.base_idx());
}

fn encode_mod(bs: &mut impl BitSetMut, range: Range<usize>, src_mod: u8) {
    assert!(range.len() == 2);
    bs.set_field(range, src_mod);
}

fn encode_cb(bs: &mut impl BitSetMut, range: Range<usize>, cb: &CBufRef) {
    let mut v = bs.subset_mut(range);
    v.set_field(0..16, cb.offset);
    if let CBuf::Binding(idx) = cb.buf {
        v.set_field(16..21, idx);
    } else {
        panic!("Must be a bound constant buffer");
    }
}

fn encode_cx(bs: &mut impl BitSetMut, range: Range<usize>, cb: &CBufRef) {
    let mut v = bs.subset_mut(range);
    if let CBuf::BindlessGPR(gpr) = cb.buf {
        encode_ureg(&mut v, 0..8, gpr);
    } else {
        panic!("Must be a bound constant buffer");
    }
    assert!(cb.offset % 4 == 0);
    v.set_field(8..22, cb.offset / 4);
}

fn encode_instr_base(bs: &mut impl BitSetMut, instr: &Instr, opcode: u16) {
    bs.set_field(0..12, opcode);
    bs.set_field(12..16, 0x7_u8); /* TODO: Predicate */
    if instr.num_dsts() > 0 {
        assert!(instr.num_dsts() == 1);
        encode_reg(bs, 16..24, *instr.dst(0).as_reg().unwrap());
    }

    bs.set_field(105..109, instr.deps.delay);
    bs.set_bit(109, instr.deps.yld);
    bs.set_field(110..113, instr.deps.wr_bar().unwrap_or(7));
    bs.set_field(113..116, instr.deps.rd_bar().unwrap_or(7));
    bs.set_field(116..122, instr.deps.wt_bar_mask);
    bs.set_field(122..126, instr.deps.reuse_mask);
}

enum ALUSrc {
    Imm(Immediate),
    Reg(RegRef),
    UReg(RegRef),
    CBuf(CBufRef),
}

impl ALUSrc {
    pub fn from_src(src: &Src, comps: u8) -> ALUSrc {
        match src {
            Src::Reg(reg) => {
                assert!(reg.comps() == comps);
                match reg.file() {
                    RegFile::GPR => ALUSrc::Reg(*reg),
                    RegFile::UGPR => ALUSrc::UReg(*reg),
                    _ => panic!("Invalid ALU register file"),
                }
            }
            Src::Imm(i) => ALUSrc::Imm(*i),
            Src::CBuf(cb) => ALUSrc::CBuf(*cb),
            _ => panic!("Invalid ALU source"),
        }
    }
}

fn encode_alu(bs: &mut impl BitSetMut, instr: &Instr, opcode: u16) {
    encode_instr_base(bs, instr, opcode);

    encode_reg(bs, 24..32, *instr.src(0).as_reg().unwrap());
    bs.set_field(32..72, 0u64); /* Pad */

    let mut form = 0_u8;
    if instr.num_srcs() > 1 {
        match ALUSrc::from_src(instr.src(1), 1) {
            ALUSrc::Reg(reg1) => {
                if instr.num_srcs() == 2 {
                    form = 1;
                    encode_reg(bs, 32..40, reg1);
                    encode_mod(bs, 62..64, instr.src_mod(1));
                } else {
                    assert!(instr.num_srcs() == 3);
                    match ALUSrc::from_src(instr.src(2), 1) {
                        ALUSrc::Reg(reg2) => {
                            form = 1;
                            encode_reg(bs, 32..40, reg1);
                            encode_mod(bs, 62..64, instr.src_mod(1));
                            encode_reg(bs, 64..72, reg2);
                            encode_mod(bs, 74..76, instr.src_mod(2));
                        }
                        ALUSrc::UReg(reg2) => {
                            form = 7;
                            encode_ureg(bs, 32..40, reg2);
                            encode_mod(bs, 62..64, instr.src_mod(2));
                            encode_reg(bs, 64..72, reg1);
                            encode_mod(bs, 74..76, instr.src_mod(1));
                        }
                        ALUSrc::Imm(imm) => {
                            form = 2;
                            encode_imm(bs, 32..64, &imm);
                            encode_reg(bs, 64..72, reg1);
                            encode_mod(bs, 74..76, instr.src_mod(1));
                        }
                        ALUSrc::CBuf(cb) => {
                            form = 3;
                            /* TODO encode_cx */
                            encode_cb(bs, 38..59, &cb);
                            encode_mod(bs, 62..64, instr.src_mod(2));
                            encode_reg(bs, 64..72, reg1);
                            encode_mod(bs, 74..76, instr.src_mod(1));
                        }
                        _ => panic!("Invalid instruction form"),
                    }
                }
            }
            ALUSrc::Reg(reg1) => {
                form = 6;
                encode_ureg(bs, 32..40, reg1);
                encode_mod(bs, 62..64, instr.src_mod(1));
                if instr.num_srcs() > 2 {
                    assert!(instr.num_srcs() == 3);
                    encode_reg(bs, 64..72, *instr.src(2).as_reg().unwrap());
                    encode_mod(bs, 74..76, instr.src_mod(2));
                }
            }
            ALUSrc::Imm(imm) => {
                form = 4;
                encode_imm(bs, 32..64, &imm);
                if instr.num_srcs() > 2 {
                    encode_reg(bs, 64..72, *instr.src(2).as_reg().unwrap());
                    encode_mod(bs, 74..76, instr.src_mod(2));
                }
            }
            ALUSrc::CBuf(cb) => {
                form = 5;
                /* TODO encode_cx */
                encode_cb(bs, 38..59, &cb);
                encode_mod(bs, 62..64, instr.src_mod(1));
                if instr.num_srcs() > 2 {
                    encode_reg(bs, 64..72, *instr.src(2).as_reg().unwrap());
                    encode_mod(bs, 74..76, instr.src_mod(2));
                }
            }
            _ => panic!("Invalid instruction form"),
        }
    }

    bs.set_field(9..12, form);
}

fn encode_mov(bs: &mut impl BitSetMut, instr: &Instr) {
    encode_alu(bs, instr, 0x002);
    bs.set_field(72..76, 0xf_u32 /* TODO: Quad lanes */);
}

fn encode_ald(bs: &mut impl BitSetMut, instr: &Instr, attr: &AttrAccess) {
    encode_instr_base(bs, &instr, 0x321);

    encode_reg(bs, 24..32, *instr.src(0).as_reg().unwrap());
    encode_reg(bs, 32..40, *instr.src(1).as_reg().unwrap());

    bs.set_field(40..50, attr.addr);
    bs.set_field(74..76, attr.comps - 1);
    bs.set_field(76..77, attr.patch);
    bs.set_field(77..78, attr.flags);
    bs.set_field(79..80, attr.out_load);
}

fn encode_ast(bs: &mut impl BitSetMut, instr: &Instr, attr: &AttrAccess) {
    encode_instr_base(bs, &instr, 0x322);
    assert!(instr.num_dsts() == 0);

    encode_reg(bs, 32..40, *instr.src(0).as_reg().unwrap());
    encode_reg(bs, 24..32, *instr.src(1).as_reg().unwrap());
    encode_reg(bs, 64..72, *instr.src(2).as_reg().unwrap());

    bs.set_field(40..50, attr.addr);
    bs.set_field(74..76, attr.comps - 1);
    bs.set_field(76..77, attr.patch);
    bs.set_field(77..78, attr.flags);
    assert!(!attr.out_load);
}

fn encode_exit(bs: &mut impl BitSetMut, instr: &Instr) {
    encode_instr_base(bs, instr, 0x94d);

    bs.set_field(84..85, false); /* ./.KEEPREFCOUNT/.PREEMPTED/.INVALID3 */
    bs.set_field(85..86, false); /* .NO_ATEXIT */
    bs.set_field(87..90, 0x7_u8); /* TODO: Predicate */
    bs.set_field(90..91, false); /* NOT */
}

pub fn encode_instr(instr: &Instr) -> [u32; 4] {
    let mut enc = [0_u32; 4];
    let mut bs = BitSetMutView::new(&mut enc);
    match &instr.op {
        Opcode::MOV => encode_mov(&mut bs, instr),
        Opcode::ALD(a) => encode_ald(&mut bs, instr, &a),
        Opcode::AST(a) => encode_ast(&mut bs, instr, &a),
        Opcode::EXIT => encode_exit(&mut bs, instr),
        _ => panic!("Unhandled instruction"),
    }
    enc
}

pub fn encode_shader(shader: &Shader) -> Vec<u32> {
    let mut encoded = Vec::new();
    assert!(shader.functions.len() == 1);
    let func = &shader.functions[0];
    for b in &func.blocks {
        for instr in &b.instrs {
            let e = encode_instr(&instr);
            encoded.extend_from_slice(&e[..]);
        }
    }
    encoded
}