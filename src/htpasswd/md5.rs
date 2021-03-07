pub(crate) const APR1_ID: &str = "$apr1$";

const DIGEST_SIZE: usize = 16;

const S11: u32 = 7;
const S12: u32 = 12;
const S13: u32 = 17;
const S14: u32 = 22;
const S21: u32 = 5;
const S22: u32 = 9;
const S23: u32 = 14;
const S24: u32 = 20;
const S31: u32 = 4;
const S32: u32 = 11;
const S33: u32 = 16;
const S34: u32 = 23;
const S41: u32 = 6;
const S42: u32 = 10;
const S43: u32 = 15;
const S44: u32 = 21;

static PADDING: [u8; 64] = [
    0x80, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0,
];

fn ff(a: &mut u32, b: u32, c: u32, d: u32, x: u32, s: u32, ac: u32) {
    *a = a
        .wrapping_add((b & c) | ((!b) & d))
        .wrapping_add(x)
        .wrapping_add(ac);
    *a = a.rotate_left(s);
    *a = a.wrapping_add(b);
}

fn gg(a: &mut u32, b: u32, c: u32, d: u32, x: u32, s: u32, ac: u32) {
    *a = a
        .wrapping_add((b & d) | (c & (!d)))
        .wrapping_add(x)
        .wrapping_add(ac);
    *a = a.rotate_left(s);
    *a = a.wrapping_add(b);
}

fn hh(a: &mut u32, b: u32, c: u32, d: u32, x: u32, s: u32, ac: u32) {
    *a = a.wrapping_add(b ^ c ^ d).wrapping_add(x).wrapping_add(ac);
    *a = a.rotate_left(s);
    *a = a.wrapping_add(b);
}

fn ii(a: &mut u32, b: u32, c: u32, d: u32, x: u32, s: u32, ac: u32) {
    *a = a
        .wrapping_add(c ^ (b | (!d)))
        .wrapping_add(x)
        .wrapping_add(ac);
    *a = a.rotate_left(s);
    *a = a.wrapping_add(b);
}

struct MD5Ctx {
    state: [u32; 4],
    count: [u32; 2],
    buffer: [u8; 64],
}

impl MD5Ctx {
    fn new() -> Self {
        MD5Ctx {
            state: [0x67452301, 0xefcdab89, 0x98badcfe, 0x10325476],
            count: [0, 0],
            buffer: [0; 64],
        }
    }

    fn init(&mut self) {
        self.state = [0x67452301, 0xefcdab89, 0x98badcfe, 0x10325476];
        self.count = [0, 0];
    }

    fn update_buffer(&mut self, input: &[u8], input_len: usize) {
        let mut i;
        let mut idx = ((self.count[0] >> 3) & 0x3F) as u32;

        self.count[0] += (input_len << 3) as u32;
        if self.count[0] < ((input_len << 3) as u32) {
            self.count[1] += 1;
        }
        self.count[1] += (input_len >> 29) as u32;

        let part_len = 64 - idx as usize;

        if input_len >= part_len {
            self.buffer[(idx as usize)..].copy_from_slice(&input[0..part_len]);
            md5_transform(&mut self.state, &self.buffer);

            i = part_len;
            while i + 63 < input_len {
                md5_transform(&mut self.state, &input[i..]);
                i += 64;
            }

            idx = 0;
        } else {
            i = 0;
        }

        if input_len - i > 0 {
            let copy_len = input_len - i;
            self.buffer[(idx as usize)..(idx as usize + copy_len)]
                .copy_from_slice(&input[i..(i + copy_len)]);
        }
    }

    fn md5_final(&mut self, digest: &mut [u8; DIGEST_SIZE]) {
        let mut bits: [u8; 8] = [0; 8];

        encode(&mut bits, &self.count, 8);

        let idx: u32 = (self.count[0] >> 3) & 0x3f;
        let pad_len = if idx < 56 { 56 - idx } else { 120 - idx } as usize;
        self.update_buffer(&PADDING, pad_len);

        self.update_buffer(&bits, bits.len());

        encode(digest, &self.state, DIGEST_SIZE);

        self.state = [0; 4];
        self.count = [0, 0];
        self.buffer = [0; 64];
    }
}

fn md5_transform(state: &mut [u32; 4], block: &[u8]) {
    let (mut a, mut b, mut c, mut d) = (state[0], state[1], state[2], state[3]);

    let x = unsafe { std::slice::from_raw_parts(block.as_ptr() as *const u32, block.len() / 4) };

    /* Round 1 */
    ff(&mut a, b, c, d, x[0], S11, 0xd76aa478); /* 1 */
    ff(&mut d, a, b, c, x[1], S12, 0xe8c7b756); /* 2 */
    ff(&mut c, d, a, b, x[2], S13, 0x242070db); /* 3 */
    ff(&mut b, c, d, a, x[3], S14, 0xc1bdceee); /* 4 */
    ff(&mut a, b, c, d, x[4], S11, 0xf57c0faf); /* 5 */
    ff(&mut d, a, b, c, x[5], S12, 0x4787c62a); /* 6 */
    ff(&mut c, d, a, b, x[6], S13, 0xa8304613); /* 7 */
    ff(&mut b, c, d, a, x[7], S14, 0xfd469501); /* 8 */
    ff(&mut a, b, c, d, x[8], S11, 0x698098d8); /* 9 */
    ff(&mut d, a, b, c, x[9], S12, 0x8b44f7af); /* 10 */
    ff(&mut c, d, a, b, x[10], S13, 0xffff5bb1); /* 11 */
    ff(&mut b, c, d, a, x[11], S14, 0x895cd7be); /* 12 */
    ff(&mut a, b, c, d, x[12], S11, 0x6b901122); /* 13 */
    ff(&mut d, a, b, c, x[13], S12, 0xfd987193); /* 14 */
    ff(&mut c, d, a, b, x[14], S13, 0xa679438e); /* 15 */
    ff(&mut b, c, d, a, x[15], S14, 0x49b40821); /* 16 */

    /* Round 2 */
    gg(&mut a, b, c, d, x[1], S21, 0xf61e2562); /* 17 */
    gg(&mut d, a, b, c, x[6], S22, 0xc040b340); /* 18 */
    gg(&mut c, d, a, b, x[11], S23, 0x265e5a51); /* 19 */
    gg(&mut b, c, d, a, x[0], S24, 0xe9b6c7aa); /* 20 */
    gg(&mut a, b, c, d, x[5], S21, 0xd62f105d); /* 21 */
    gg(&mut d, a, b, c, x[10], S22, 0x2441453); /* 22 */
    gg(&mut c, d, a, b, x[15], S23, 0xd8a1e681); /* 23 */
    gg(&mut b, c, d, a, x[4], S24, 0xe7d3fbc8); /* 24 */
    gg(&mut a, b, c, d, x[9], S21, 0x21e1cde6); /* 25 */
    gg(&mut d, a, b, c, x[14], S22, 0xc33707d6); /* 26 */
    gg(&mut c, d, a, b, x[3], S23, 0xf4d50d87); /* 27 */
    gg(&mut b, c, d, a, x[8], S24, 0x455a14ed); /* 28 */
    gg(&mut a, b, c, d, x[13], S21, 0xa9e3e905); /* 29 */
    gg(&mut d, a, b, c, x[2], S22, 0xfcefa3f8); /* 30 */
    gg(&mut c, d, a, b, x[7], S23, 0x676f02d9); /* 31 */
    gg(&mut b, c, d, a, x[12], S24, 0x8d2a4c8a); /* 32 */

    /* Round 3 */
    hh(&mut a, b, c, d, x[5], S31, 0xfffa3942); /* 33 */
    hh(&mut d, a, b, c, x[8], S32, 0x8771f681); /* 34 */
    hh(&mut c, d, a, b, x[11], S33, 0x6d9d6122); /* 35 */
    hh(&mut b, c, d, a, x[14], S34, 0xfde5380c); /* 36 */
    hh(&mut a, b, c, d, x[1], S31, 0xa4beea44); /* 37 */
    hh(&mut d, a, b, c, x[4], S32, 0x4bdecfa9); /* 38 */
    hh(&mut c, d, a, b, x[7], S33, 0xf6bb4b60); /* 39 */
    hh(&mut b, c, d, a, x[10], S34, 0xbebfbc70); /* 40 */
    hh(&mut a, b, c, d, x[13], S31, 0x289b7ec6); /* 41 */
    hh(&mut d, a, b, c, x[0], S32, 0xeaa127fa); /* 42 */
    hh(&mut c, d, a, b, x[3], S33, 0xd4ef3085); /* 43 */
    hh(&mut b, c, d, a, x[6], S34, 0x4881d05); /* 44 */
    hh(&mut a, b, c, d, x[9], S31, 0xd9d4d039); /* 45 */
    hh(&mut d, a, b, c, x[12], S32, 0xe6db99e5); /* 46 */
    hh(&mut c, d, a, b, x[15], S33, 0x1fa27cf8); /* 47 */
    hh(&mut b, c, d, a, x[2], S34, 0xc4ac5665); /* 48 */

    /* Round 4 */
    ii(&mut a, b, c, d, x[0], S41, 0xf4292244); /* 49 */
    ii(&mut d, a, b, c, x[7], S42, 0x432aff97); /* 50 */
    ii(&mut c, d, a, b, x[14], S43, 0xab9423a7); /* 51 */
    ii(&mut b, c, d, a, x[5], S44, 0xfc93a039); /* 52 */
    ii(&mut a, b, c, d, x[12], S41, 0x655b59c3); /* 53 */
    ii(&mut d, a, b, c, x[3], S42, 0x8f0ccc92); /* 54 */
    ii(&mut c, d, a, b, x[10], S43, 0xffeff47d); /* 55 */
    ii(&mut b, c, d, a, x[1], S44, 0x85845dd1); /* 56 */
    ii(&mut a, b, c, d, x[8], S41, 0x6fa87e4f); /* 57 */
    ii(&mut d, a, b, c, x[15], S42, 0xfe2ce6e0); /* 58 */
    ii(&mut c, d, a, b, x[6], S43, 0xa3014314); /* 59 */
    ii(&mut b, c, d, a, x[13], S44, 0x4e0811a1); /* 60 */
    ii(&mut a, b, c, d, x[4], S41, 0xf7537e82); /* 61 */
    ii(&mut d, a, b, c, x[11], S42, 0xbd3af235); /* 62 */
    ii(&mut c, d, a, b, x[2], S43, 0x2ad7d2bb); /* 63 */
    ii(&mut b, c, d, a, x[9], S44, 0xeb86d391); /* 64 */

    state[0] = state[0].wrapping_add(a);
    state[1] = state[1].wrapping_add(b);
    state[2] = state[2].wrapping_add(c);
    state[3] = state[3].wrapping_add(d);
}

fn encode(output: &mut [u8], input: &[u32], len: usize) {
    for (i, j) in (0..len).step_by(4).enumerate() {
        let k = input[i];
        output[j] = (k & 0xff) as u8;
        output[j + 1] = ((k >> 8) & 0xff) as u8;
        output[j + 2] = ((k >> 16) & 0xff) as u8;
        output[j + 3] = ((k >> 24) & 0xff) as u8;
    }
}

#[allow(dead_code)]
fn decode(output: &mut [u32], input: &[u8], len: usize) {
    for (i, j) in (0..len).step_by(4).enumerate() {
        output[i] = input[j] as u32
            | ((input[j + 1] as u32) << 8)
            | ((input[j + 2] as u32) << 16)
            | ((input[j + 3] as u32) << 24);
    }
}

fn encode_digest(digest: &[u32; 16]) -> String {
    let mut p = vec![0u8; 22];
    let l = ((digest[0] << 16) | (digest[6] << 8) | digest[12]) as u64;
    to_64(&mut p[0..4], l, 4);

    let l = ((digest[1] << 16) | (digest[7] << 8) | digest[13]) as u64;
    to_64(&mut p[4..8], l, 4);

    let l = ((digest[2] << 16) | (digest[8] << 8) | digest[14]) as u64;
    to_64(&mut p[8..12], l, 4);

    let l = ((digest[3] << 16) | (digest[9] << 8) | digest[15]) as u64;
    to_64(&mut p[12..16], l, 4);

    let l = ((digest[4] << 16) | (digest[10] << 8) | digest[5]) as u64;
    to_64(&mut p[16..20], l, 4);

    let l = digest[11] as u64;
    to_64(&mut p[20..22], l, 2);

    String::from_utf8(p).unwrap()
}

fn to_64(s: &mut [u8], mut v: u64, n: i32) {
    let itoa64 = "./0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz".as_bytes();

    for tmp in s.iter_mut().take(n as usize) {
        *tmp = itoa64[(v & 0x3f) as usize];
        v >>= 6;
    }
}

/// Calculates apache specific md5 hash
/// Returns just the hashed password, use [format_hash](fn.format_hash.html) to get the hash in htpasswd format
pub fn md5_apr1_encode(pw: &str, salt: &str) -> Option<String> {
    if salt.len() != 8 {
        return None;
    }

    let mut sp = salt.as_bytes();
    let pw = pw.as_bytes();

    if sp.starts_with(APR1_ID.as_bytes()) {
        sp = &sp[APR1_ID.len()..sp.len()];
    }

    let mut ctx = MD5Ctx::new();
    ctx.update_buffer(pw, pw.len());
    ctx.update_buffer(APR1_ID.as_bytes(), APR1_ID.len());
    ctx.update_buffer(sp, sp.len());

    let mut ctx1 = MD5Ctx::new();
    ctx1.update_buffer(pw, pw.len());
    ctx1.update_buffer(sp, sp.len());
    ctx1.update_buffer(pw, pw.len());

    let mut digest = [0u8; DIGEST_SIZE];
    ctx1.md5_final(&mut digest);

    for pl in (1..(pw.len() + 1)).rev().step_by(DIGEST_SIZE) {
        ctx.update_buffer(&digest, if pl > DIGEST_SIZE { DIGEST_SIZE } else { pl });
    }

    digest = [0u8; DIGEST_SIZE];

    let mut i = pw.len();
    while i != 0 {
        if i & 1 != 0 {
            ctx.update_buffer(&digest, 1);
        } else {
            ctx.update_buffer(pw, 1);
        }
        i >>= 1;
    }

    ctx.md5_final(&mut digest);

    for i in 0u32..1000u32 {
        ctx1.init();

        if i & 1 != 0 {
            ctx1.update_buffer(pw, pw.len());
        } else {
            ctx1.update_buffer(&digest, DIGEST_SIZE);
        }
        if i % 3 != 0 {
            ctx1.update_buffer(sp, sp.len());
        }

        if i % 7 != 0 {
            ctx1.update_buffer(pw, pw.len());
        }

        if i & 1 != 0 {
            ctx1.update_buffer(&digest, DIGEST_SIZE);
        } else {
            ctx1.update_buffer(pw, pw.len());
        }
        ctx1.md5_final(&mut digest);
    }

    let mut digest_final: [u32; 16] = [0; 16];
    digest
        .iter()
        .enumerate()
        .for_each(|(idx, &x)| digest_final[idx] = x as u32);

    Some(encode_digest(&digest_final))
}

pub fn format_hash(password: &str, salt: &str) -> String {
    format!("{}{}${}", APR1_ID, salt, password)
}

/// Assumes the hash is in the correct format - $apr1$salt$password
pub fn verify_apr1_hash(hash: &str, password: &str) -> Result<bool, &'static str> {
    let salt = &hash[6..14];
    Ok(format_hash(&md5_apr1_encode(password, salt).unwrap(), salt) == hash)
}
