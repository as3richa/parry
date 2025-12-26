use crate::field::Field;
use std::fmt;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq)]
pub(crate) struct Gf8(pub u8);

impl Gf8 {
    const EXP: [u8; 255] = [
        1u8, 3u8, 5u8, 15u8, 17u8, 51u8, 85u8, 255u8, 26u8, 46u8, 114u8, 150u8, 161u8, 248u8, 19u8,
        53u8, 95u8, 225u8, 56u8, 72u8, 216u8, 115u8, 149u8, 164u8, 247u8, 2u8, 6u8, 10u8, 30u8,
        34u8, 102u8, 170u8, 229u8, 52u8, 92u8, 228u8, 55u8, 89u8, 235u8, 38u8, 106u8, 190u8, 217u8,
        112u8, 144u8, 171u8, 230u8, 49u8, 83u8, 245u8, 4u8, 12u8, 20u8, 60u8, 68u8, 204u8, 79u8,
        209u8, 104u8, 184u8, 211u8, 110u8, 178u8, 205u8, 76u8, 212u8, 103u8, 169u8, 224u8, 59u8,
        77u8, 215u8, 98u8, 166u8, 241u8, 8u8, 24u8, 40u8, 120u8, 136u8, 131u8, 158u8, 185u8, 208u8,
        107u8, 189u8, 220u8, 127u8, 129u8, 152u8, 179u8, 206u8, 73u8, 219u8, 118u8, 154u8, 181u8,
        196u8, 87u8, 249u8, 16u8, 48u8, 80u8, 240u8, 11u8, 29u8, 39u8, 105u8, 187u8, 214u8, 97u8,
        163u8, 254u8, 25u8, 43u8, 125u8, 135u8, 146u8, 173u8, 236u8, 47u8, 113u8, 147u8, 174u8,
        233u8, 32u8, 96u8, 160u8, 251u8, 22u8, 58u8, 78u8, 210u8, 109u8, 183u8, 194u8, 93u8, 231u8,
        50u8, 86u8, 250u8, 21u8, 63u8, 65u8, 195u8, 94u8, 226u8, 61u8, 71u8, 201u8, 64u8, 192u8,
        91u8, 237u8, 44u8, 116u8, 156u8, 191u8, 218u8, 117u8, 159u8, 186u8, 213u8, 100u8, 172u8,
        239u8, 42u8, 126u8, 130u8, 157u8, 188u8, 223u8, 122u8, 142u8, 137u8, 128u8, 155u8, 182u8,
        193u8, 88u8, 232u8, 35u8, 101u8, 175u8, 234u8, 37u8, 111u8, 177u8, 200u8, 67u8, 197u8,
        84u8, 252u8, 31u8, 33u8, 99u8, 165u8, 244u8, 7u8, 9u8, 27u8, 45u8, 119u8, 153u8, 176u8,
        203u8, 70u8, 202u8, 69u8, 207u8, 74u8, 222u8, 121u8, 139u8, 134u8, 145u8, 168u8, 227u8,
        62u8, 66u8, 198u8, 81u8, 243u8, 14u8, 18u8, 54u8, 90u8, 238u8, 41u8, 123u8, 141u8, 140u8,
        143u8, 138u8, 133u8, 148u8, 167u8, 242u8, 13u8, 23u8, 57u8, 75u8, 221u8, 124u8, 132u8,
        151u8, 162u8, 253u8, 28u8, 36u8, 108u8, 180u8, 199u8, 82u8, 246u8,
    ];

    const LOG: [u8; 256] = [
        255u8, 0u8, 25u8, 1u8, 50u8, 2u8, 26u8, 198u8, 75u8, 199u8, 27u8, 104u8, 51u8, 238u8,
        223u8, 3u8, 100u8, 4u8, 224u8, 14u8, 52u8, 141u8, 129u8, 239u8, 76u8, 113u8, 8u8, 200u8,
        248u8, 105u8, 28u8, 193u8, 125u8, 194u8, 29u8, 181u8, 249u8, 185u8, 39u8, 106u8, 77u8,
        228u8, 166u8, 114u8, 154u8, 201u8, 9u8, 120u8, 101u8, 47u8, 138u8, 5u8, 33u8, 15u8, 225u8,
        36u8, 18u8, 240u8, 130u8, 69u8, 53u8, 147u8, 218u8, 142u8, 150u8, 143u8, 219u8, 189u8,
        54u8, 208u8, 206u8, 148u8, 19u8, 92u8, 210u8, 241u8, 64u8, 70u8, 131u8, 56u8, 102u8, 221u8,
        253u8, 48u8, 191u8, 6u8, 139u8, 98u8, 179u8, 37u8, 226u8, 152u8, 34u8, 136u8, 145u8, 16u8,
        126u8, 110u8, 72u8, 195u8, 163u8, 182u8, 30u8, 66u8, 58u8, 107u8, 40u8, 84u8, 250u8, 133u8,
        61u8, 186u8, 43u8, 121u8, 10u8, 21u8, 155u8, 159u8, 94u8, 202u8, 78u8, 212u8, 172u8, 229u8,
        243u8, 115u8, 167u8, 87u8, 175u8, 88u8, 168u8, 80u8, 244u8, 234u8, 214u8, 116u8, 79u8,
        174u8, 233u8, 213u8, 231u8, 230u8, 173u8, 232u8, 44u8, 215u8, 117u8, 122u8, 235u8, 22u8,
        11u8, 245u8, 89u8, 203u8, 95u8, 176u8, 156u8, 169u8, 81u8, 160u8, 127u8, 12u8, 246u8,
        111u8, 23u8, 196u8, 73u8, 236u8, 216u8, 67u8, 31u8, 45u8, 164u8, 118u8, 123u8, 183u8,
        204u8, 187u8, 62u8, 90u8, 251u8, 96u8, 177u8, 134u8, 59u8, 82u8, 161u8, 108u8, 170u8, 85u8,
        41u8, 157u8, 151u8, 178u8, 135u8, 144u8, 97u8, 190u8, 220u8, 252u8, 188u8, 149u8, 207u8,
        205u8, 55u8, 63u8, 91u8, 209u8, 83u8, 57u8, 132u8, 60u8, 65u8, 162u8, 109u8, 71u8, 20u8,
        42u8, 158u8, 93u8, 86u8, 242u8, 211u8, 171u8, 68u8, 17u8, 146u8, 217u8, 35u8, 32u8, 46u8,
        137u8, 180u8, 124u8, 184u8, 38u8, 119u8, 153u8, 227u8, 165u8, 103u8, 74u8, 237u8, 222u8,
        197u8, 49u8, 254u8, 24u8, 13u8, 99u8, 140u8, 128u8, 192u8, 247u8, 112u8, 7u8,
    ];

    pub fn elements() -> Box<[Gf8]> {
        (0u8..=255u8)
            .map(Gf8)
            .collect::<Vec<Gf8>>()
            .into_boxed_slice()
    }
}

impl fmt::Debug for Gf8 {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "Gf8(0x{:02x})", self.0)
    }
}

impl Field for Gf8 {
    fn zero() -> Gf8 {
        Gf8(0u8)
    }

    fn one() -> Gf8 {
        Gf8(1u8)
    }
}

impl Add for Gf8 {
    type Output = Gf8;

    fn add(self, y: Gf8) -> Gf8 {
        Gf8(self.0 ^ y.0)
    }
}

impl AddAssign for Gf8 {
    fn add_assign(&mut self, y: Gf8) {
        *self = *self + y;
    }
}

impl Sub for Gf8 {
    type Output = Gf8;

    fn sub(self, y: Gf8) -> Gf8 {
        Gf8(self.0 ^ y.0)
    }
}

impl SubAssign for Gf8 {
    fn sub_assign(&mut self, y: Gf8) {
        *self = *self - y;
    }
}

impl Mul for Gf8 {
    type Output = Gf8;

    fn mul(self, y: Gf8) -> Gf8 {
        if self.0 == 0 || y.0 == 0 {
            return Gf8(0);
        }

        let x_log: u8 = Gf8::LOG[self.0 as usize];
        let y_log: u8 = Gf8::LOG[y.0 as usize];
        let z_log: usize = ((x_log as usize) + (y_log as usize)) % 255;
        Gf8(Gf8::EXP[z_log])
    }
}

impl MulAssign for Gf8 {
    fn mul_assign(&mut self, y: Gf8) {
        *self = *self * y;
    }
}

impl Div for Gf8 {
    type Output = Gf8;

    fn div(self, y: Gf8) -> Gf8 {
        if self.0 == 0 {
            return Gf8(0);
        }

        if y.0 == 0 {
            panic!("Gf8 division by zero");
        }

        let x_log: u8 = Gf8::LOG[self.0 as usize];
        let y_log: u8 = Gf8::LOG[y.0 as usize];
        let z_log: usize = ((x_log as usize) + 255 - (y_log as usize)) % 255;
        Gf8(Gf8::EXP[z_log])
    }
}

impl DivAssign for Gf8 {
    fn div_assign(&mut self, y: Gf8) {
        *self = *self / y;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_commutative() {
        for x in Gf8::elements() {
            for y in Gf8::elements() {
                assert!(x + y == y + x);
            }
        }
    }

    #[test]
    fn add_associative() {
        for x in Gf8::elements() {
            for y in Gf8::elements() {
                for z in Gf8::elements() {
                    assert!((x + y) + z == x + (y + z));
                }
            }
        }
    }

    #[test]
    fn add_assign() {
        for x in Gf8::elements() {
            for y in Gf8::elements() {
                let mut x_mut = x;
                x_mut += y;
                assert!(x_mut == x + y);
            }
        }
    }

    #[test]
    fn add_equivalent_to_sub() {
        for x in Gf8::elements() {
            for y in Gf8::elements() {
                assert!(x + y == x - y);
            }
        }
    }

    fn russian_peasant_mul(x: Gf8, y: Gf8) -> Gf8 {
        let mut x0: usize = x.0 as usize;
        let mut y0: usize = y.0 as usize;
        let mut z: usize = 0;

        while x0 > 0 && y0 > 0 {
            if (y0 & 1) == 1 {
                z ^= x0;
            }

            if (x0 & 0x80) != 0 {
                x0 = (x0 << 1) ^ 0x11b;
            } else {
                x0 <<= 1;
            }

            y0 >>= 1;
        }

        Gf8(z as u8)
    }

    #[test]
    fn exp_log() {
        let g: Gf8 = Gf8(0b11);
        let mut g_exp: Gf8 = Gf8(0b1);

        for i in 0..255 {
            assert!(Gf8(Gf8::EXP[i]) == g_exp);
            assert!(Gf8::LOG[g_exp.0 as usize] == i as u8);

            g_exp = russian_peasant_mul(g_exp, g);
        }
    }

    #[test]
    fn mul() {
        for x in Gf8::elements() {
            for y in Gf8::elements() {
                assert!(x * y == russian_peasant_mul(x, y));
            }
        }
    }

    #[test]
    fn mul_zero() {
        for x in Gf8::elements() {
            assert!(x * Gf8::zero() == Gf8::zero());
            assert!(Gf8::zero() * x == Gf8::zero());
        }
    }

    #[test]
    fn mul_one() {
        for x in Gf8::elements() {
            assert!(x * Gf8::one() == x);
            assert!(Gf8::one() * x == x);
        }
    }

    #[test]
    fn mul_commutative() {
        for x in Gf8::elements() {
            for y in Gf8::elements() {
                assert!(x * y == y * x);
            }
        }
    }

    #[test]
    fn mul_associative() {
        for x in Gf8::elements() {
            for y in Gf8::elements() {
                for z in Gf8::elements() {
                    assert!((x * y) * z == x * (y * z));
                }
            }
        }
    }

    #[test]
    fn mul_distributive() {
        for x in Gf8::elements() {
            for y in Gf8::elements() {
                for z in Gf8::elements() {
                    assert!(x * (y + z) == x * y + x * z);
                }
            }
        }
    }

    #[test]
    fn div_undoes_mul() {
        for x in Gf8::elements() {
            for y in Gf8::elements() {
                if y == Gf8::zero() {
                    continue;
                }

                assert!(x * y / y == x);
            }
        }
    }

    #[test]
    fn div_associative_with_mul() {
        for x in Gf8::elements() {
            for y in Gf8::elements() {
                for z in Gf8::elements() {
                    if z == Gf8::zero() {
                        continue;
                    }

                    assert!((x * y) / z == x * (y / z));
                }
            }
        }
    }
}
