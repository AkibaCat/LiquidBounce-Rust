// SPDX-License-Identifier: GPL-3.0-or-later
//! Pinned NameGenerator, MD5(name UTF-8) big-endian Long and Kotlin Random(Long).
use std::sync::OnceLock;
const ADJECTIVES:&str=include_str!("../data/name_protect/adjectives.txt");
const ANIMALS:&str=include_str!("../data/name_protect/animals.txt");
struct Words { adjectives:Vec<&'static str>, animals:Vec<&'static str> }
fn words()->&'static Words {
    static WORDS:OnceLock<Words>=OnceLock::new();
    WORDS.get_or_init(||{
        fn load(text:&'static str)->Vec<&'static str>{let mut words=text.lines().collect::<Vec<_>>();words.sort_by_key(|s|s.len());words}
        Words{adjectives:load(ADJECTIVES),animals:load(ANIMALS)}
    })
}
struct KotlinRandom { x:u32,y:u32,z:u32,w:u32,v:u32,addend:u32 }
impl KotlinRandom {
    fn new(seed:u64)->Self {
        let x=seed as u32;let y=(seed>>32)as u32;
        let mut r=Self{x,y,z:0,w:0,v:!x,addend:(x<<10)^(y>>4)};
        for _ in 0..64{r.int();}r
    }
    fn int(&mut self)->u32 {
        let mut t=self.x^(self.x>>2);self.x=self.y;self.y=self.z;self.z=self.w;
        let v=self.v;self.w=v;t=t^(t<<1)^v^(v<<4);self.v=t;
        self.addend=self.addend.wrapping_add(362437);t.wrapping_add(self.addend)
    }
    fn bits(&mut self,count:u32)->u32 {
        // Kotlin nextBits(0) still consumes one random integer.
        let n=self.int();if count==0{0}else{n>>(32-count)}
    }
    fn bounded(&mut self,bound:u32)->usize {
        assert!((1..=i32::MAX as u32).contains(&bound));
        if bound.is_power_of_two(){return self.bits(bound.ilog2())as usize;}
        loop {
            let n=self.int()>>1;let value=n%bound;
            if n.wrapping_sub(value).wrapping_add(bound-1)as i32>=0{return value as usize;}
        }
    }
}
pub fn pseudonym(name:&str)->String {
    let digest=md5(name.as_bytes());
    let mut rng=KotlinRandom::new(u64::from_be_bytes(digest[..8].try_into().unwrap()));
    let words=words();
    let (first,second)=if rng.bits(1)!=0{(&words.adjectives,&words.animals)}else{(&words.animals,&words.adjectives)};
    let choose=|words:&Vec<&'static str>,limit:usize,rng:&mut KotlinRandom|{
        let count=words.partition_point(|s|s.len()<=limit);words[rng.bounded(count as u32)]
    };
    let a=choose(first,13,&mut rng);let b=choose(second,16-a.len(),&mut rng);
    let mut elements=vec![a.to_owned(),b.to_owned()];let len=a.len()+b.len();
    if len+1<16&&rng.bounded(20)!=0{
        let until=(16-len).min(3);
        let digits=if until<=2{until}else{2+rng.bounded((until-2)as u32)};
        elements.push(rng.bounded(10u32.pow(digits as u32)).to_string());
    }
    let allowed=16-elements.iter().map(String::len).sum::<usize>();
    let mut delimiters=rng.bits(2);
    while delimiters.count_ones()as usize>allowed{delimiters=rng.bits(2);}
    let mut out=elements[0].clone();
    for e in elements.iter().skip(1){if delimiters&1!=0{out.push('_');}delimiters>>=1;out.push_str(e);}
    let count=rng.bounded(3);
    let mut swaps=out.bytes().enumerate().filter_map(|(i,c)|{
        let replacement=match c{b'a'=>b'4',b'b'=>b'8',b'e'=>b'3',b'g'=>b'6',b'i'=>b'1',b'o'=>b'0',b's'=>b'5',b't'=>b'7',b'z'=>b'2',_=>return None};
        Some((i,replacement))
    }).collect::<Vec<_>>();
    // CollectionsKt.shuffle(List, Random): descending Fisher-Yates, including nextInt(2).
    for i in (1..swaps.len()).rev(){let j=rng.bounded((i+1)as u32);swaps.swap(i,j);}
    let mut bytes=out.into_bytes();
    for &(i,c) in swaps.iter().take(count){bytes[i]=c;}
    String::from_utf8(bytes).expect("ASCII username resources")
}
fn md5(input:&[u8])->[u8;16] {
    const S:[u32;64]=[
        7,12,17,22,7,12,17,22,7,12,17,22,7,12,17,22,
        5,9,14,20,5,9,14,20,5,9,14,20,5,9,14,20,
        4,11,16,23,4,11,16,23,4,11,16,23,4,11,16,23,
        6,10,15,21,6,10,15,21,6,10,15,21,6,10,15,21];
    const K:[u32;64]=[
        0xd76aa478,0xe8c7b756,0x242070db,0xc1bdceee,0xf57c0faf,0x4787c62a,0xa8304613,0xfd469501,
        0x698098d8,0x8b44f7af,0xffff5bb1,0x895cd7be,0x6b901122,0xfd987193,0xa679438e,0x49b40821,
        0xf61e2562,0xc040b340,0x265e5a51,0xe9b6c7aa,0xd62f105d,0x02441453,0xd8a1e681,0xe7d3fbc8,
        0x21e1cde6,0xc33707d6,0xf4d50d87,0x455a14ed,0xa9e3e905,0xfcefa3f8,0x676f02d9,0x8d2a4c8a,
        0xfffa3942,0x8771f681,0x6d9d6122,0xfde5380c,0xa4beea44,0x4bdecfa9,0xf6bb4b60,0xbebfbc70,
        0x289b7ec6,0xeaa127fa,0xd4ef3085,0x04881d05,0xd9d4d039,0xe6db99e5,0x1fa27cf8,0xc4ac5665,
        0xf4292244,0x432aff97,0xab9423a7,0xfc93a039,0x655b59c3,0x8f0ccc92,0xffeff47d,0x85845dd1,
        0x6fa87e4f,0xfe2ce6e0,0xa3014314,0x4e0811a1,0xf7537e82,0xbd3af235,0x2ad7d2bb,0xeb86d391];
    let mut bytes=input.to_vec();let len=(input.len()as u64).wrapping_mul(8);bytes.push(0x80);
    while bytes.len()%64!=56{bytes.push(0);}bytes.extend_from_slice(&len.to_le_bytes());
    let mut state=[0x67452301u32,0xefcdab89,0x98badcfe,0x10325476];
    for block in bytes.chunks_exact(64){
        let mut words=[0u32;16];for (i,c) in block.chunks_exact(4).enumerate(){words[i]=u32::from_le_bytes(c.try_into().unwrap());}
        let [mut a,mut b,mut c,mut d]=state;
        for i in 0..64{
            let (f,g)=match i{0..=15=>((b&c)|(!b&d),i),16..=31=>((d&b)|(!d&c),(5*i+1)%16),32..=47=>(b^c^d,(3*i+5)%16),_=>(c^(b|!d),(7*i)%16)};
            let next=b.wrapping_add(a.wrapping_add(f).wrapping_add(K[i]).wrapping_add(words[g]).rotate_left(S[i]));
            a=d;d=c;c=b;b=next;
        }
        for (s,v) in state.iter_mut().zip([a,b,c,d]){*s=s.wrapping_add(v);}
    }
    let mut out=[0u8;16];for (s,c) in state.iter().zip(out.chunks_exact_mut(4)){c.copy_from_slice(&s.to_le_bytes());}out
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]fn digest_uses_utf8_and_standard_md5_vectors(){
        let hex=|b:[u8;16]|b.iter().map(|v|format!("{v:02x}")).collect::<String>();
        assert_eq!(hex(md5(b"")),"d41d8cd98f00b204e9800998ecf8427e");
        assert_eq!(hex(md5(b"abc")),"900150983cd24fb0d6963f7d28e17f72");
        assert_eq!(hex(md5("玩家😀".as_bytes())),"026a2c2a1265c75d9febb6d92246604f");
    }
    #[test]fn deterministic_ascii_names_respect_maximum_length(){
        for name in ["Alice","Bob","Notch","玩家😀","_","","abcdefghijklmnopqrst"]{
            let output=pseudonym(name);assert_eq!(output,pseudonym(name));assert!(output.len()<=16);
            assert!(output.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'_'));
        }
    }
}
