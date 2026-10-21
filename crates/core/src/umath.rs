//! Integer geometry of MOO1 (1oom `util_math.c`): angles from tangent tables, movement in
//! steps of 5 and 6 map units (one parsec is 11 units), distances in parsec steps.

const TAN_0: [u16; 22] = [
    0x00, 0x04, 0x08, 0x0d, 0x11, 0x16, 0x1a, 0x1f, 0x23, 0x28, 0x2d, 0x31, 0x36, 0x3b, 0x3f, 0x44,
    0x49, 0x4e, 0x53, 0x58, 0x5d, 0x62,
];
const TAN_22: [u16; 23] = [
    0x67, 0x6c, 0x71, 0x77, 0x7c, 0x82, 0x88, 0x8d, 0x93, 0x99, 0x9f, 0xa6, 0xac, 0xb3, 0xb9, 0xc0,
    0xc8, 0xcf, 0xd6, 0xde, 0xe6, 0xee, 0xf7,
];
const TAN_45: [u16; 22] = [
    0x0100, 0x0109, 0x0112, 0x011c, 0x0126, 0x0131, 0x013c, 0x0147, 0x0153, 0x0160, 0x016d, 0x017b,
    0x018a, 0x0199, 0x01aa, 0x01bb, 0x01cd, 0x01e1, 0x01f6, 0x020c, 0x0224, 0x023e,
];
const TAN_67: [u16; 23] = [
    0x025b, 0x0279, 0x029a, 0x02bf, 0x02e7, 0x0313, 0x0345, 0x037c, 0x03bb, 0x0402, 0x0454, 0x04b4,
    0x0525, 0x05ab, 0x0650, 0x071d, 0x0825, 0x0983, 0x0b6e, 0x0e4d, 0x1315, 0x1ca3, 0x394d,
];
const SIN: [u32; 90] = [
    0x0000, 0x0477, 0x08ef, 0x0d65, 0x11db, 0x164f, 0x1ac2, 0x1f32, 0x23a0, 0x280c, 0x2c74, 0x30d8,
    0x3539, 0x3996, 0x3dee, 0x4242, 0x4690, 0x4ad8, 0x4f1b, 0x5358, 0x578e, 0x5bbe, 0x5fe6, 0x6407,
    0x681f, 0x6c30, 0x7039, 0x7438, 0x782f, 0x7c1c, 0x8000, 0x83d9, 0x87a8, 0x8b6d, 0x8f27, 0x92d5,
    0x9679, 0x9a10, 0x9d9c, 0xa11b, 0xa48d, 0xa7f3, 0xab4c, 0xae97, 0xb1d5, 0xb505, 0xb826, 0xbb3a,
    0xbe3e, 0xc134, 0xc41b, 0xc6f3, 0xc9bb, 0xcc73, 0xcf1b, 0xd1b4, 0xd43b, 0xd6b3, 0xd919, 0xdb6f,
    0xddb3, 0xdfe7, 0xe208, 0xe419, 0xe617, 0xe803, 0xe9de, 0xeba6, 0xed5b, 0xeeff, 0xf08f, 0xf20d,
    0xf378, 0xf4d0, 0xf615, 0xf746, 0xf865, 0xf970, 0xfa67, 0xfb4b, 0xfc1c, 0xfcd9, 0xfd82, 0xfe17,
    0xfe99, 0xff06, 0xff60, 0xffa6, 0xffd8, 0xfff6,
];
const COS: [u32; 90] = [
    0xfff6, 0xffd8, 0xffa6, 0xff60, 0xff06, 0xfe98, 0xfe17, 0xfd82, 0xfcd9, 0xfc1c, 0xfb4b, 0xfa67,
    0xf970, 0xf865, 0xf746, 0xf615, 0xf4d0, 0xf378, 0xf20d, 0xf08f, 0xeeff, 0xed5b, 0xeba6, 0xe9de,
    0xe803, 0xe617, 0xe418, 0xe208, 0xdfe7, 0xddb3, 0xdb6f, 0xd919, 0xd6b3, 0xd43b, 0xd1b3, 0xcf1b,
    0xcc73, 0xc9bb, 0xc6f2, 0xc41b, 0xc134, 0xbe3e, 0xbb39, 0xb826, 0xb504, 0xb1d5, 0xae97, 0xab4c,
    0xa7f3, 0xa48d, 0xa11b, 0x9d9b, 0x9a10, 0x9678, 0x92d5, 0x8f27, 0x8b6d, 0x87a8, 0x83d9, 0x7fff,
    0x7c1c, 0x782f, 0x7438, 0x7038, 0x6c30, 0x681f, 0x6406, 0x5fe6, 0x5bbd, 0x578e, 0x5358, 0x4f1b,
    0x4ad8, 0x468f, 0x4241, 0x3dee, 0x3996, 0x3539, 0x30d8, 0x2c73, 0x280b, 0x23a0, 0x1f32, 0x1ac2,
    0x164f, 0x11db, 0x0d65, 0x08ee, 0x0477, 0x0000,
];

/// The index where the first table value greater than `slope` is, 1oom style.
fn scan(tbl: &[u16], slope: u32) -> Option<i32> {
    tbl.iter().position(|&t| slope < t as u32).map(|i| i as i32)
}

fn calc_angle_do(dx: u32, dy: u32) -> i32 {
    if dx == 0 {
        return 90;
    }
    let slope = (dy << 8) / dx;
    if slope < 0x100 {
        if slope >= 0x67 {
            22 + scan(&TAN_22, slope).map_or(21, |i| i - 1)
        } else {
            scan(&TAN_0, slope).unwrap_or(0x16) - 1
        }
    } else if slope >= 0x258 {
        match scan(&TAN_67, slope) {
            Some(i) => i - 1 + 67,
            None => 90,
        }
    } else {
        match scan(&TAN_45, slope) {
            Some(i) => i - 1 + 45,
            None => 66,
        }
    }
}

/// 1oom `util_math_calc_angle`: 0..359, 0 = right, 90 = down.
pub fn calc_angle(dx: i32, dy: i32) -> i32 {
    if dx >= 0 {
        if dy >= 0 {
            calc_angle_do(dx as u32, dy as u32)
        } else {
            360 - calc_angle_do(dx as u32, (-dy) as u32)
        }
    } else if dy >= 0 {
        180 - calc_angle_do((-dx) as u32, dy as u32)
    } else {
        180 + calc_angle_do((-dx) as u32, (-dy) as u32)
    }
}

pub fn angle_dist_cos(angle: i32, dist: i32) -> i32 {
    let (a, neg) = if angle < 91 {
        (angle, false)
    } else if angle < 181 {
        (180 - angle, true)
    } else if angle < 271 {
        (angle - 180, true)
    } else {
        (360 - angle, false)
    };
    let res = if a == 0 {
        dist
    } else {
        ((dist as i64 * COS[(a - 1) as usize] as i64) / 0x10000) as i32
    };
    if neg {
        -res
    } else {
        res
    }
}

pub fn angle_dist_sin(angle: i32, dist: i32) -> i32 {
    let (a, neg) = if angle < 91 {
        (angle, false)
    } else if angle < 181 {
        (180 - angle, false)
    } else if angle < 271 {
        (angle - 180, true)
    } else {
        (360 - angle, true)
    };
    let res = if a == 90 {
        dist
    } else {
        ((dist as i64 * SIN[a as usize] as i64) / 0x10000) as i32
    };
    if neg {
        -res
    } else {
        res
    }
}

/// 1oom `util_math_go_line_dist`: moves (x, y) `dist` units towards (x1, y1).
pub fn go_line_dist(x: &mut i32, y: &mut i32, x1: i32, y1: i32, dist: i32) {
    if dist <= 0 || (*x == x1 && *y == y1) {
        return;
    }
    let (mut dx, mut dy) = (x1 - *x, y1 - *y);
    if dx.abs() > 0xff || dy.abs() > 0xff {
        dx /= 2;
        dy /= 2;
    }
    let angle = calc_angle(dx, dy);
    let mut nx = *x + angle_dist_cos(angle, dist);
    if (dx >= 0 && nx > x1) || (dx < 0 && nx < x1) {
        nx = x1;
    }
    let mut ny = *y + angle_dist_sin(angle, dist);
    if (dy >= 0 && ny > y1) || (dy < 0 && ny < y1) {
        ny = y1;
    }
    *x = nx;
    *y = ny;
}

/// 1oom `util_math_dist_steps`: parsecs (5 + 6 unit steps) between two points.
pub fn dist_steps(x0: i32, y0: i32, x1: i32, y1: i32) -> i32 {
    let (mut x, mut y) = (x0, y0);
    let mut num = 0;
    while x != x1 || y != y1 {
        go_line_dist(&mut x, &mut y, x1, y1, 5);
        if x != x1 || y != y1 {
            go_line_dist(&mut x, &mut y, x1, y1, 6);
        }
        num += 1;
    }
    num
}

/// 1oom `util_math_dist_fast`.
pub fn dist_fast(x0: i32, y0: i32, x1: i32, y1: i32) -> i32 {
    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    if dx > dy {
        dx + dy / 2
    } else {
        dy + dx / 2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn angles() {
        assert_eq!(calc_angle(10, 0), 0);
        assert_eq!(calc_angle(0, 10), 90);
        assert_eq!(calc_angle(-10, 0), 180);
        assert_eq!(calc_angle(0, -10), 270);
        assert_eq!(calc_angle(10, 10), 45);
        // Values printed by 1oom's util_math.c.
        let (mut x, mut y) = (0, 0);
        go_line_dist(&mut x, &mut y, 30, 40, 5);
        assert_eq!((x, y), (3, 3));
        assert_eq!(dist_fast(0, 0, 30, 40), 55);
    }

    #[test]
    fn matches_1oom_tables() {
        assert_eq!(calc_angle(39, -1), 359);
        assert_eq!(calc_angle(57, -1), 359);
        assert_eq!(dist_steps(14, 13, 71, 12), 7);
        // util_math_calc_angle(dx, dy) printed by 1oom for dx in -60..=60 step 7, dy in
        // -60..=60 step 11.
        const WANT: [i32; 198] = [
            225, 219, 212, 204, 195, 184, 175, 165, 155, 147, 141, 228, 222, 215, 207, 196, 185,
            174, 163, 153, 144, 137, 232, 226, 219, 210, 199, 186, 173, 160, 149, 140, 133, 236,
            231, 223, 214, 202, 187, 172, 157, 145, 135, 128, 241, 236, 229, 220, 206, 189, 170,
            152, 139, 130, 123, 247, 242, 236, 227, 212, 191, 167, 146, 132, 123, 117, 253, 249,
            244, 236, 221, 195, 162, 137, 123, 115, 110, 259, 257, 253, 247, 235, 204, 152, 123,
            112, 106, 103, 266, 265, 263, 261, 255, 231, 124, 104, 99, 96, 95, 273, 274, 275, 277,
            281, 301, 63, 79, 83, 85, 86, 280, 282, 285, 291, 302, 334, 31, 59, 70, 75, 78, 286,
            290, 295, 303, 317, 344, 19, 45, 58, 66, 71, 292, 297, 303, 312, 327, 349, 14, 35, 49,
            58, 64, 298, 303, 310, 319, 333, 351, 11, 28, 42, 51, 58, 303, 308, 315, 325, 338, 353,
            9, 24, 36, 45, 52, 307, 313, 320, 329, 341, 354, 7, 20, 32, 40, 48, 311, 317, 324, 333,
            343, 355, 6, 18, 28, 37, 43, 315, 321, 328, 336, 345, 356, 6, 16, 25, 33, 40,
        ];
        let mut got = Vec::new();
        for dx in (-60..=60).step_by(7) {
            for dy in (-60..=60).step_by(11) {
                got.push(calc_angle(dx, dy));
            }
        }
        assert_eq!(got, WANT);
    }

    #[test]
    fn steps() {
        assert_eq!(dist_steps(0, 0, 11, 0), 1);
        assert_eq!(dist_steps(0, 0, 12, 0), 2);
        let (mut x, mut y) = (0, 0);
        go_line_dist(&mut x, &mut y, 100, 0, 5);
        assert_eq!((x, y), (5, 0));
    }
}
