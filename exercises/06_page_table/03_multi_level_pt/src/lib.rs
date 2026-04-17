//! # SV39 涓夌骇椤佃〃
//!
//! 鏈粌涔犳ā鎷?RISC-V SV39 涓夌骇椤佃〃鐨勬瀯閫犲拰鍦板潃缈昏瘧銆?
//! 娉ㄦ剰锛屽疄闄呬笂鐨勪笁绾ч〉琛ㄥ疄鐜板苟闈炲鏈粌涔犱腑浣跨敤 HashMap 妯℃嫙锛屾湰缁冧範浠呬綔涓烘ā鎷熷府鍔╁涔犮€?
//! 浣犻渶瑕佸疄鐜伴〉琛ㄧ殑鍒涘缓銆佹槧灏勫拰鍦板潃缈昏瘧锛堥〉琛ㄩ亶鍘嗭級銆?
//!
//! ## 鐭ヨ瘑鐐?
//! - SV39锛?9 浣嶈櫄鎷熷湴鍧€锛屼笁绾ч〉琛?
//! - VPN 鎷嗗垎锛歏PN[2] (9bit) | VPN[1] (9bit) | VPN[0] (9bit)
//! - 椤佃〃閬嶅巻锛坧age table walk锛夐€愮骇鏌ユ壘
//! - 澶ч〉锛?MB superpage锛夋槧灏?
//!
//! ## SV39 铏氭嫙鍦板潃甯冨眬
//! ```text
//! 38        30 29       21 20       12 11        0
//! 鈹屸攢鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹攢鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹攢鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹攢鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹?
//! 鈹?VPN[2]   鈹? VPN[1]   鈹? VPN[0]   鈹? offset   鈹?
//! 鈹? 9 bits  鈹? 9 bits   鈹? 9 bits   鈹? 12 bits  鈹?
//! 鈹斺攢鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹粹攢鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹粹攢鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹粹攢鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹?
//! ```

use std::collections::HashMap;

/// 椤靛ぇ灏?4KB
pub const PAGE_SIZE: usize = 4096;
/// 姣忕骇椤佃〃鏈?512 涓潯鐩?(2^9)
pub const PT_ENTRIES: usize = 512;

/// PTE 鏍囧織浣?
pub const PTE_V: u64 = 1 << 0;
pub const PTE_R: u64 = 1 << 1;
pub const PTE_W: u64 = 1 << 2;
pub const PTE_X: u64 = 1 << 3;

/// PPN 鍦?PTE 涓殑鍋忕Щ
const PPN_SHIFT: u32 = 10;

/// 椤佃〃鑺傜偣锛氫竴涓寘鍚?512 涓潯鐩殑鏁扮粍
#[derive(Clone)]
pub struct PageTableNode {
    pub entries: [u64; PT_ENTRIES],
}

impl PageTableNode {
    pub fn new() -> Self {
        Self {
            entries: [0; PT_ENTRIES],
        }
    }
}

impl Default for PageTableNode {
    fn default() -> Self {
        Self::new()
    }
}

/// 妯℃嫙鐨勪笁绾ч〉琛ㄣ€?
///
/// 浣跨敤 HashMap<u64, PageTableNode> 妯℃嫙鐗╃悊鍐呭瓨涓殑椤佃〃椤点€?
/// `root_ppn` 鏄牴椤佃〃鎵€鍦ㄧ殑鐗╃悊椤靛彿銆?
pub struct Sv39PageTable {
    /// 鐗╃悊椤靛彿 -> 椤佃〃鑺傜偣
    nodes: HashMap<u64, PageTableNode>,
    /// 鏍归〉琛ㄧ殑鐗╃悊椤靛彿
    pub root_ppn: u64,
    /// 涓嬩竴涓彲鍒嗛厤鐨勭墿鐞嗛〉鍙凤紙绠€鏄撳垎閰嶅櫒锛?
    next_ppn: u64,
}

/// 缈昏瘧缁撴灉
#[derive(Debug, PartialEq)]
pub enum TranslateResult {
    Ok(u64),
    PageFault,
}

impl Sv39PageTable {
    pub fn new() -> Self {
        let mut pt = Self {
            nodes: HashMap::new(),
            root_ppn: 0x80000,
            next_ppn: 0x80001,
        };
        pt.nodes.insert(pt.root_ppn, PageTableNode::new());
        pt
    }

    /// 鍒嗛厤涓€涓柊鐨勭墿鐞嗛〉骞跺垵濮嬪寲涓虹┖椤佃〃鑺傜偣锛岃繑鍥炲叾 PPN銆?
    fn alloc_node(&mut self) -> u64 {
        let ppn = self.next_ppn;
        self.next_ppn += 1;
        self.nodes.insert(ppn, PageTableNode::new());
        ppn
    }

    /// 浠?39 浣嶈櫄鎷熷湴鍧€涓彁鍙栫 `level` 绾х殑 VPN銆?
    ///
    /// - level=2: 鍙?bits [38:30]
    /// - level=1: 鍙?bits [29:21]
    /// - level=0: 鍙?bits [20:12]
    ///
    /// 鎻愮ず锛氬彸绉?(12 + level * 9) 浣嶏紝鐒跺悗涓?0x1FF 鍋氭帺鐮併€?
    pub fn extract_vpn(va: u64, level: usize) -> usize {
        ((va >> (12 + level * 9)) & 0x1FF) as usize
    }

    /// 寤虹珛浠庤櫄鎷熼〉鍒扮墿鐞嗛〉鐨勬槧灏勶紙4KB 椤碉級銆?
    ///
    /// 鍙傛暟锛?
    /// - `va`: 铏氭嫙鍦板潃锛堜細鑷姩瀵归綈鍒伴〉杈圭晫锛?
    /// - `pa`: 鐗╃悊鍦板潃锛堜細鑷姩瀵归綈鍒伴〉杈圭晫锛?
    /// - `flags`: 鏍囧織浣嶏紙濡?PTE_V | PTE_R | PTE_W锛?
    pub fn map_page(&mut self, va: u64, pa: u64, flags: u64) {
        let mut ppn = self.root_ppn;
        for level in [2usize, 1usize] {
            let idx = Self::extract_vpn(va, level);
            let entry = {
                let node = self.nodes.get(&ppn).unwrap();
                node.entries[idx]
            };

            if entry & PTE_V == 0 || entry & (PTE_R | PTE_W | PTE_X) != 0 {
                let new_ppn = self.alloc_node();
                let node = self.nodes.get_mut(&ppn).unwrap();
                node.entries[idx] = (new_ppn << PPN_SHIFT) | PTE_V;
            }

            ppn = {
                let node = self.nodes.get(&ppn).unwrap();
                node.entries[idx] >> PPN_SHIFT
            };
        }

        let idx = Self::extract_vpn(va, 0);
        let node = self.nodes.get_mut(&ppn).unwrap();
        node.entries[idx] = ((pa >> 12) << PPN_SHIFT) | flags;
    }

    /// 閬嶅巻涓夌骇椤佃〃锛屽皢铏氭嫙鍦板潃缈昏瘧涓虹墿鐞嗗湴鍧€銆?
    ///
    /// 姝ラ锛?
    /// 1. 浠庢牴椤佃〃锛坮oot_ppn锛夊紑濮?
    /// 2. 瀵规瘡涓€绾э紙2, 1, 0锛夛細
    ///    a. 鐢?VPN[level] 绱㈠紩褰撳墠椤佃〃鑺傜偣
    ///    b. 濡傛灉 PTE 鏃犳晥锛?PTE_V锛夛紝杩斿洖 PageFault
    ///    c. 濡傛灉 PTE 鏄彾鑺傜偣锛圧|W|X 鏈変换涓€缃綅锛夛紝鎻愬彇 PPN 璁＄畻鐗╃悊鍦板潃
    ///    d. 鍚﹀垯鐢?PTE 涓殑 PPN 杩涘叆涓嬩竴绾ч〉琛?
    /// 3. level 0 鐨?PTE 蹇呴』鏄彾鑺傜偣
    pub fn translate(&self, va: u64) -> TranslateResult {
        let mut ppn = self.root_ppn;
        for level in [2usize, 1usize, 0usize] {
            let idx = Self::extract_vpn(va, level);
            let entry = match self.nodes.get(&ppn) {
                Some(node) => node.entries[idx],
                None => return TranslateResult::PageFault,
            };

            if entry & PTE_V == 0 {
                return TranslateResult::PageFault;
            }

            if entry & (PTE_R | PTE_W | PTE_X) != 0 {
                let offset_mask = match level {
                    2 => (1u64 << 30) - 1,
                    1 => (1u64 << 21) - 1,
                    _ => (1u64 << 12) - 1,
                };
                let pa = ((entry >> PPN_SHIFT) << 12) | (va & offset_mask);
                return TranslateResult::Ok(pa);
            }

            if level == 0 {
                return TranslateResult::PageFault;
            }

            ppn = entry >> PPN_SHIFT;
        }

        TranslateResult::PageFault
    }

    /// 寤虹珛澶ч〉鏄犲皠锛?MB superpage锛屽湪 level 1 璁惧彾瀛?PTE锛夈€?
    ///
    /// 2MB = 512 脳 4KB锛屽榻愯姹傦細va 鍜?pa 閮藉繀椤?2MB 瀵归綈銆?
    ///
    /// 涓?map_page 绫讳技锛屼絾鍙亶鍘嗗埌 level 1 灏卞啓鍏ュ彾瀛?PTE銆?
    pub fn map_superpage(&mut self, va: u64, pa: u64, flags: u64) {
        let mega_size: u64 = (PAGE_SIZE * PT_ENTRIES) as u64; // 2MB
        assert_eq!(va % mega_size, 0, "va must be 2MB-aligned");
        assert_eq!(pa % mega_size, 0, "pa must be 2MB-aligned");

        let vpn2 = Self::extract_vpn(va, 2);
        let entry = {
            let node = self.nodes.get(&self.root_ppn).unwrap();
            node.entries[vpn2]
        };

        if entry & PTE_V == 0 || entry & (PTE_R | PTE_W | PTE_X) != 0 {
            let new_ppn = self.alloc_node();
            let root = self.nodes.get_mut(&self.root_ppn).unwrap();
            root.entries[vpn2] = (new_ppn << PPN_SHIFT) | PTE_V;
        }

        let ppn = {
            let root = self.nodes.get(&self.root_ppn).unwrap();
            root.entries[vpn2] >> PPN_SHIFT
        };

        let vpn1 = Self::extract_vpn(va, 1);
        let node = self.nodes.get_mut(&ppn).unwrap();
        node.entries[vpn1] = ((pa >> 12) << PPN_SHIFT) | flags;
    }
}

impl Default for Sv39PageTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_vpn() {
        // VA = 0x0000_003F_FFFF_F000 (鏈€澶х殑 39 浣嶅湴鍧€鐨勯〉杈圭晫)
        // VPN[2] = 0xFF (bits 38:30)
        // VPN[1] = 0x1FF (bits 29:21)
        // VPN[0] = 0x1FF (bits 20:12)
        let va: u64 = 0x7FFFFFF000;
        assert_eq!(Sv39PageTable::extract_vpn(va, 2), 0x1FF);
        assert_eq!(Sv39PageTable::extract_vpn(va, 1), 0x1FF);
        assert_eq!(Sv39PageTable::extract_vpn(va, 0), 0x1FF);
    }

    #[test]
    fn test_extract_vpn_simple() {
        // VA = 0x00000000 + page 1 = 0x1000
        // VPN[2] = 0, VPN[1] = 0, VPN[0] = 1
        let va: u64 = 0x1000;
        assert_eq!(Sv39PageTable::extract_vpn(va, 2), 0);
        assert_eq!(Sv39PageTable::extract_vpn(va, 1), 0);
        assert_eq!(Sv39PageTable::extract_vpn(va, 0), 1);
    }

    #[test]
    fn test_extract_vpn_level2() {
        // VPN[2] = 1 means bit 30 set -> VA >= 0x40000000
        let va: u64 = 0x40000000;
        assert_eq!(Sv39PageTable::extract_vpn(va, 2), 1);
        assert_eq!(Sv39PageTable::extract_vpn(va, 1), 0);
        assert_eq!(Sv39PageTable::extract_vpn(va, 0), 0);
    }

    #[test]
    fn test_map_and_translate_single() {
        let mut pt = Sv39PageTable::new();
        // 鏄犲皠锛歏A 0x1000 -> PA 0x80001000
        pt.map_page(0x1000, 0x80001000, PTE_V | PTE_R);

        let result = pt.translate(0x1000);
        assert_eq!(result, TranslateResult::Ok(0x80001000));
    }

    #[test]
    fn test_translate_with_offset() {
        let mut pt = Sv39PageTable::new();
        pt.map_page(0x2000, 0x90000000, PTE_V | PTE_R | PTE_W);

        // 璁块棶 VA 0x2ABC -> PA 搴斾负 0x90000ABC
        let result = pt.translate(0x2ABC);
        assert_eq!(result, TranslateResult::Ok(0x90000ABC));
    }

    #[test]
    fn test_translate_page_fault() {
        let pt = Sv39PageTable::new();
        assert_eq!(pt.translate(0x1000), TranslateResult::PageFault);
    }

    #[test]
    fn test_multiple_mappings() {
        let mut pt = Sv39PageTable::new();
        pt.map_page(0x0000_1000, 0x8000_1000, PTE_V | PTE_R);
        pt.map_page(0x0000_2000, 0x8000_5000, PTE_V | PTE_R | PTE_W);
        pt.map_page(0x0040_0000, 0x9000_0000, PTE_V | PTE_R);

        assert_eq!(pt.translate(0x1234), TranslateResult::Ok(0x80001234));
        assert_eq!(pt.translate(0x2000), TranslateResult::Ok(0x80005000));
        assert_eq!(pt.translate(0x400100), TranslateResult::Ok(0x90000100));
    }

    #[test]
    fn test_map_overwrite() {
        let mut pt = Sv39PageTable::new();
        pt.map_page(0x1000, 0x80001000, PTE_V | PTE_R);
        assert_eq!(pt.translate(0x1000), TranslateResult::Ok(0x80001000));

        pt.map_page(0x1000, 0x90002000, PTE_V | PTE_R);
        assert_eq!(pt.translate(0x1000), TranslateResult::Ok(0x90002000));
    }

    #[test]
    fn test_superpage_mapping() {
        let mut pt = Sv39PageTable::new();
        // 2MB 澶ч〉鏄犲皠锛歏A 0x200000 -> PA 0x80200000
        pt.map_superpage(0x200000, 0x80200000, PTE_V | PTE_R | PTE_W);

        // 澶ч〉鍐呬笉鍚屽亸绉婚兘搴斿懡涓?
        assert_eq!(pt.translate(0x200000), TranslateResult::Ok(0x80200000));
        assert_eq!(pt.translate(0x200ABC), TranslateResult::Ok(0x80200ABC));
        assert_eq!(pt.translate(0x2FF000), TranslateResult::Ok(0x802FF000));
    }

    #[test]
    fn test_superpage_and_normal_coexist() {
        let mut pt = Sv39PageTable::new();
        // 澶ч〉鏄犲皠鍦ㄧ涓€涓?2MB 鍖哄煙
        pt.map_superpage(0x0, 0x80000000, PTE_V | PTE_R);
        // 鏅€氶〉鍦ㄤ笉鍚岀殑 VPN[2] 鍖哄煙
        pt.map_page(0x40000000, 0x90001000, PTE_V | PTE_R);

        assert_eq!(pt.translate(0x100), TranslateResult::Ok(0x80000100));
        assert_eq!(pt.translate(0x40000000), TranslateResult::Ok(0x90001000));
    }
}
