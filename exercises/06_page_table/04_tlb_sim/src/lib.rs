//! # TLB 妯℃嫙涓庡埛鏂?
//!
//! 鏈粌涔犳ā鎷?TLB锛圱ranslation Lookaside Buffer锛屽湴鍧€缈昏瘧鍚庡缂撳啿鍖猴級锛?
//! 甯姪浣犵悊瑙?TLB 鐨勬煡鎵俱€佹彃鍏ャ€佹浛鎹㈠拰鍒锋柊鏈哄埗銆?
//!
//! ## 鐭ヨ瘑鐐?
//! - TLB 鏄〉琛ㄧ殑纭欢缂撳瓨锛屽姞閫熻櫄鎷熷湴鍧€缈昏瘧
//! - TLB 鍛戒腑/鏈懡涓紙hit/miss锛?
//! - TLB 鏇挎崲绛栫暐锛堟湰缁冧範浣跨敤 FIFO锛?
//! - TLB 鍒锋柊锛氬叏閮ㄥ埛鏂般€佹寜铏氭嫙椤靛埛鏂般€佹寜 ASID 鍒锋柊
//! - ASID锛圓ddress Space Identifier锛夊尯鍒嗕笉鍚岃繘绋嬬殑鍦板潃绌洪棿
//! - MMU 宸ヤ綔娴佺▼锛氬厛鏌?TLB锛宮iss 鍒欒蛋椤佃〃锛屽啀鍥炲～ TLB
//!
//! ## TLB 鏉＄洰缁撴瀯
//! ```text
//! 鈹屸攢鈹€鈹€鈹€鈹€鈹€鈹€鈹攢鈹€鈹€鈹€鈹€鈹€鈹攢鈹€鈹€鈹€鈹€鈹€鈹攢鈹€鈹€鈹€鈹€鈹€鈹€鈹攢鈹€鈹€鈹€鈹€鈹€鈹€鈹?
//! 鈹?valid 鈹?asid 鈹?vpn  鈹? ppn  鈹?flags 鈹?
//! 鈹斺攢鈹€鈹€鈹€鈹€鈹€鈹€鈹粹攢鈹€鈹€鈹€鈹€鈹€鈹粹攢鈹€鈹€鈹€鈹€鈹€鈹粹攢鈹€鈹€鈹€鈹€鈹€鈹€鈹粹攢鈹€鈹€鈹€鈹€鈹€鈹€鈹?
//! ```

/// TLB 鏉＄洰
#[derive(Clone, Debug)]
pub struct TlbEntry {
    pub valid: bool,
    pub asid: u16,
    pub vpn: u64,
    pub ppn: u64,
    pub flags: u64,
}

impl TlbEntry {
    pub fn empty() -> Self {
        Self {
            valid: false,
            asid: 0,
            vpn: 0,
            ppn: 0,
            flags: 0,
        }
    }
}

/// TLB 缁熻淇℃伅
#[derive(Debug, Default)]
pub struct TlbStats {
    pub hits: u64,
    pub misses: u64,
}

impl TlbStats {
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }
}

/// 妯℃嫙 TLB锛屽浐瀹氬ぇ灏忥紝浣跨敤 FIFO 鏇挎崲绛栫暐銆?
pub struct Tlb {
    entries: Vec<TlbEntry>,
    capacity: usize,
    /// FIFO 鎸囬拡锛氫笅娆℃浛鎹㈢殑浣嶇疆
    fifo_ptr: usize,
    pub stats: TlbStats,
}

impl Tlb {
    /// 鍒涘缓涓€涓閲忎负 `capacity` 鐨?TLB銆?
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: vec![TlbEntry::empty(); capacity],
            capacity,
            fifo_ptr: 0,
            stats: TlbStats::default(),
        }
    }

    /// 鍦?TLB 涓煡鎵惧尮閰?`vpn` 鍜?`asid` 鐨勬潯鐩€?
    ///
    /// 鏌ユ壘瑙勫垯锛?
    /// - 閬嶅巻鎵€鏈夋潯鐩?
    /// - 鏉＄洰蹇呴』 `valid == true`
    /// - 鏉＄洰鐨?`vpn` 鍜?`asid` 閮藉繀椤诲尮閰?
    /// - 鍛戒腑鏃跺鍔?`stats.hits`锛屾湭鍛戒腑澧炲姞 `stats.misses`
    ///
    /// 杩斿洖鍖归厤鏉＄洰鐨?`ppn`锛屾湭鍛戒腑杩斿洖 None銆?
    pub fn lookup(&mut self, vpn: u64, asid: u16) -> Option<u64> {
        for entry in &self.entries {
            if entry.valid && entry.vpn == vpn && entry.asid == asid {
                self.stats.hits += 1;
                return Some(entry.ppn);
            }
        }
        self.stats.misses += 1;
        None
    }

    /// 灏嗕竴鏉℃柊鏄犲皠鎻掑叆 TLB銆?
    ///
    /// 浣跨敤 FIFO 鏇挎崲绛栫暐锛?
    /// 1. 鍏堟鏌ユ槸鍚﹀凡瀛樺湪鐩稿悓 (vpn, asid) 鐨勬湁鏁堟潯鐩紝濡傛灉鏈夊垯鏇存柊瀹?
    /// 2. 鍚﹀垯锛屽啓鍏?`fifo_ptr` 鎸囧悜鐨勪綅缃?
    /// 3. 灏?`fifo_ptr` 鍓嶈繘鍒颁笅涓€涓綅缃紙寰幆锛歚(fifo_ptr + 1) % capacity`锛?
    pub fn insert(&mut self, vpn: u64, ppn: u64, asid: u16, flags: u64) {
        if self.capacity == 0 {
            return;
        }

        for entry in &mut self.entries {
            if entry.valid && entry.vpn == vpn && entry.asid == asid {
                entry.ppn = ppn;
                entry.flags = flags;
                return;
            }
        }

        self.entries[self.fifo_ptr] = TlbEntry {
            valid: true,
            asid,
            vpn,
            ppn,
            flags,
        };
        self.fifo_ptr = (self.fifo_ptr + 1) % self.capacity;
    }

    /// 鍒锋柊鏁翠釜 TLB锛堝皢鎵€鏈夋潯鐩爣璁颁负鏃犳晥锛夈€?
    ///
    /// 杩欏搴斾簬 RISC-V 鐨?`sfence.vma`锛堜笉甯﹀弬鏁帮級鎿嶄綔銆?
    pub fn flush_all(&mut self) {
        for entry in &mut self.entries {
            entry.valid = false;
        }
    }

    /// 鍒锋柊鎸囧畾铏氭嫙椤电殑 TLB 鏉＄洰銆?
    ///
    /// 瀵瑰簲 `sfence.vma vaddr`锛氬彧鍒锋柊鍖归厤 `vpn` 鐨勬潯鐩紙浠绘剰 ASID锛夈€?
    pub fn flush_by_vpn(&mut self, vpn: u64) {
        for entry in &mut self.entries {
            if entry.valid && entry.vpn == vpn {
                entry.valid = false;
            }
        }
    }

    /// 鍒锋柊鎸囧畾鍦板潃绌洪棿锛圓SID锛夌殑鎵€鏈?TLB 鏉＄洰銆?
    ///
    /// 瀵瑰簲 `sfence.vma zero, asid`锛氬埛鏂拌 ASID 鐨勬墍鏈夋潯鐩€?
    pub fn flush_by_asid(&mut self, asid: u16) {
        for entry in &mut self.entries {
            if entry.valid && entry.asid == asid {
                entry.valid = false;
            }
        }
    }

    /// 杩斿洖褰撳墠鏈夋晥鏉＄洰鐨勬暟閲忋€?
    pub fn valid_count(&self) -> usize {
        self.entries.iter().filter(|entry| entry.valid).count()
    }
}

/// 椤佃〃椤癸紙绠€鍖栫増锛岀敤浜?MMU 妯℃嫙锛?
pub struct PageMapping {
    pub vpn: u64,
    pub ppn: u64,
    pub flags: u64,
}

/// 妯℃嫙鐨?MMU锛氬寘鍚?TLB 鍜屼竴涓畝鍗曠殑椤佃〃銆?
///
/// MMU 缈昏瘧娴佺▼锛?
/// 1. 鍏堟煡 TLB锛坙ookup锛?
/// 2. TLB 鍛戒腑 鈫?鐩存帴杩斿洖鐗╃悊椤靛彿
/// 3. TLB 鏈懡涓?鈫?閬嶅巻椤佃〃鏌ユ壘锛坵alk page table锛?
/// 4. 椤佃〃鍛戒腑 鈫?灏嗙粨鏋滃洖濉埌 TLB锛坕nsert锛夛紝鐒跺悗杩斿洖
/// 5. 椤佃〃涔熸湭鍛戒腑 鈫?缂洪〉锛圢one锛?
pub struct Mmu {
    pub tlb: Tlb,
    /// 绠€鍖栫殑椤佃〃锛?vpn, asid) -> PageMapping
    page_table: Vec<(u16, PageMapping)>,
    pub current_asid: u16,
}

impl Mmu {
    pub fn new(tlb_capacity: usize) -> Self {
        Self {
            tlb: Tlb::new(tlb_capacity),
            page_table: Vec::new(),
            current_asid: 0,
        }
    }

    /// 鍦ㄩ〉琛ㄤ腑娣诲姞涓€鏉℃槧灏勩€?
    pub fn add_mapping(&mut self, asid: u16, vpn: u64, ppn: u64, flags: u64) {
        self.page_table
            .push((asid, PageMapping { vpn, ppn, flags }));
    }

    /// 鍒囨崲褰撳墠鍦板潃绌洪棿锛圓SID锛夈€?
    pub fn switch_asid(&mut self, new_asid: u16) {
        self.current_asid = new_asid;
    }

    /// 妯℃嫙 MMU 鍦板潃缈昏瘧銆?
    ///
    /// 娴佺▼锛?
    /// 1. 浣跨敤 `self.current_asid` 鍜?`vpn` 鏌ユ壘 TLB
    /// 2. TLB 鍛戒腑 鈫?杩斿洖 Some(ppn)
    /// 3. TLB 鏈懡涓?鈫?鍦?`self.page_table` 涓煡鎵惧尮閰?(current_asid, vpn) 鐨勬潯鐩?
    /// 4. 椤佃〃鍛戒腑 鈫?鍥炲～ TLB锛坕nsert锛夛紝杩斿洖 Some(ppn)
    /// 5. 椤佃〃鏈懡涓?鈫?杩斿洖 None锛堢己椤碉級
    pub fn translate(&mut self, vpn: u64) -> Option<u64> {
        if let Some(ppn) = self.tlb.lookup(vpn, self.current_asid) {
            return Some(ppn);
        }

        if let Some((_, mapping)) = self
            .page_table
            .iter()
            .find(|(asid, mapping)| *asid == self.current_asid && mapping.vpn == vpn)
        {
            let vpn = mapping.vpn;
            let ppn = mapping.ppn;
            let flags = mapping.flags;
            self.tlb.insert(vpn, ppn, self.current_asid, flags);
            Some(ppn)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€ TLB 鍩虹娴嬭瘯 鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€

    #[test]
    fn test_tlb_empty_lookup() {
        let mut tlb = Tlb::new(4);
        assert_eq!(tlb.lookup(0x100, 0), None);
        assert_eq!(tlb.stats.misses, 1);
        assert_eq!(tlb.stats.hits, 0);
    }

    #[test]
    fn test_tlb_insert_and_lookup() {
        let mut tlb = Tlb::new(4);
        tlb.insert(0x100, 0x200, 1, 0x7);
        assert_eq!(tlb.lookup(0x100, 1), Some(0x200));
        assert_eq!(tlb.stats.hits, 1);
    }

    #[test]
    fn test_tlb_asid_isolation() {
        let mut tlb = Tlb::new(4);
        tlb.insert(0x100, 0x200, 1, 0x7);
        tlb.insert(0x100, 0x300, 2, 0x7);

        // 鍚屼竴 VPN锛屼笉鍚?ASID 搴旇繑鍥炰笉鍚?PPN
        assert_eq!(tlb.lookup(0x100, 1), Some(0x200));
        assert_eq!(tlb.lookup(0x100, 2), Some(0x300));
    }

    #[test]
    fn test_tlb_miss_wrong_asid() {
        let mut tlb = Tlb::new(4);
        tlb.insert(0x100, 0x200, 1, 0x7);

        // ASID 涓嶅尮閰嶅簲璇?miss
        assert_eq!(tlb.lookup(0x100, 99), None);
        assert_eq!(tlb.stats.misses, 1);
    }

    #[test]
    fn test_tlb_fifo_eviction() {
        let mut tlb = Tlb::new(2); // 鍙湁 2 涓Ы浣?
        tlb.insert(0x10, 0x20, 0, 0x7);
        tlb.insert(0x30, 0x40, 0, 0x7);
        // TLB 婊′簡锛屽啀鎻掑叆搴旇娣樻卑鏈€鍏堟彃鍏ョ殑
        tlb.insert(0x50, 0x60, 0, 0x7);

        // 0x10 搴旇琚窐姹?
        assert_eq!(tlb.lookup(0x10, 0), None);
        // 0x30 鍜?0x50 搴旇杩樺湪
        assert_eq!(tlb.lookup(0x30, 0), Some(0x40));
        assert_eq!(tlb.lookup(0x50, 0), Some(0x60));
    }

    #[test]
    fn test_tlb_update_existing() {
        let mut tlb = Tlb::new(4);
        tlb.insert(0x100, 0x200, 1, 0x3);
        tlb.insert(0x100, 0x999, 1, 0x7); // 鏇存柊鍚屼竴鏉＄洰

        assert_eq!(tlb.lookup(0x100, 1), Some(0x999));
        assert_eq!(tlb.valid_count(), 1); // 涓嶅簲璇ュ鍑轰竴鏉?
    }

    #[test]
    fn test_tlb_valid_count() {
        let mut tlb = Tlb::new(4);
        assert_eq!(tlb.valid_count(), 0);

        tlb.insert(0x1, 0x10, 0, 0x7);
        assert_eq!(tlb.valid_count(), 1);

        tlb.insert(0x2, 0x20, 0, 0x7);
        assert_eq!(tlb.valid_count(), 2);
    }

    // 鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€ TLB 鍒锋柊娴嬭瘯 鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€

    #[test]
    fn test_flush_all() {
        let mut tlb = Tlb::new(4);
        tlb.insert(0x1, 0x10, 0, 0x7);
        tlb.insert(0x2, 0x20, 1, 0x7);
        tlb.insert(0x3, 0x30, 2, 0x7);
        assert_eq!(tlb.valid_count(), 3);

        tlb.flush_all();
        assert_eq!(tlb.valid_count(), 0);
        assert_eq!(tlb.lookup(0x1, 0), None);
    }

    #[test]
    fn test_flush_by_vpn() {
        let mut tlb = Tlb::new(4);
        tlb.insert(0x100, 0x200, 1, 0x7);
        tlb.insert(0x100, 0x300, 2, 0x7); // 鍚?VPN 涓嶅悓 ASID
        tlb.insert(0x999, 0x400, 1, 0x7);

        tlb.flush_by_vpn(0x100);

        // VPN=0x100 鐨勪袱鏉￠兘搴旇鍒锋帀
        assert_eq!(tlb.lookup(0x100, 1), None);
        assert_eq!(tlb.lookup(0x100, 2), None);
        // VPN=0x999 涓嶅彈褰卞搷
        assert_eq!(tlb.lookup(0x999, 1), Some(0x400));
    }

    #[test]
    fn test_flush_by_asid() {
        let mut tlb = Tlb::new(4);
        tlb.insert(0x1, 0x10, 1, 0x7);
        tlb.insert(0x2, 0x20, 1, 0x7);
        tlb.insert(0x3, 0x30, 2, 0x7);

        tlb.flush_by_asid(1);

        // ASID=1 鐨勬潯鐩鍒锋帀
        assert_eq!(tlb.lookup(0x1, 1), None);
        assert_eq!(tlb.lookup(0x2, 1), None);
        // ASID=2 涓嶅彈褰卞搷
        assert_eq!(tlb.lookup(0x3, 2), Some(0x30));
    }

    #[test]
    fn test_flush_by_vpn_then_reinsert() {
        let mut tlb = Tlb::new(4);
        tlb.insert(0x100, 0x200, 1, 0x7);
        tlb.flush_by_vpn(0x100);
        assert_eq!(tlb.lookup(0x100, 1), None);

        // 閲嶆柊鎻掑叆鍚庡簲璇ヨ兘鎵惧埌
        tlb.insert(0x100, 0x500, 1, 0x7);
        assert_eq!(tlb.lookup(0x100, 1), Some(0x500));
    }

    // 鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€ MMU 闆嗘垚娴嬭瘯 鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€

    #[test]
    fn test_mmu_basic_translate() {
        let mut mmu = Mmu::new(4);
        mmu.current_asid = 1;
        mmu.add_mapping(1, 0x100, 0x200, 0x7);

        // 绗竴娆★細TLB miss锛岃蛋椤佃〃
        let ppn = mmu.translate(0x100);
        assert_eq!(ppn, Some(0x200));
        assert_eq!(mmu.tlb.stats.misses, 1);
        assert_eq!(mmu.tlb.stats.hits, 0);

        // 绗簩娆★細TLB hit
        let ppn = mmu.translate(0x100);
        assert_eq!(ppn, Some(0x200));
        assert_eq!(mmu.tlb.stats.hits, 1);
    }

    #[test]
    fn test_mmu_page_fault() {
        let mut mmu = Mmu::new(4);
        mmu.current_asid = 1;
        // 娌℃湁娣诲姞浠讳綍鏄犲皠
        assert_eq!(mmu.translate(0x999), None);
    }

    #[test]
    fn test_mmu_asid_switch() {
        let mut mmu = Mmu::new(4);
        mmu.add_mapping(1, 0x100, 0x200, 0x7);
        mmu.add_mapping(2, 0x100, 0x300, 0x7);

        mmu.switch_asid(1);
        assert_eq!(mmu.translate(0x100), Some(0x200));

        mmu.switch_asid(2);
        assert_eq!(mmu.translate(0x100), Some(0x300));
    }

    #[test]
    fn test_mmu_flush_on_asid_switch() {
        let mut mmu = Mmu::new(4);
        mmu.add_mapping(1, 0x100, 0x200, 0x7);
        mmu.add_mapping(2, 0x100, 0x300, 0x7);

        mmu.switch_asid(1);
        assert_eq!(mmu.translate(0x100), Some(0x200));

        // 鍒囨崲 ASID 鍚庡埛鏂?TLB 涓棫 ASID 鐨勬潯鐩?
        mmu.switch_asid(2);
        mmu.tlb.flush_by_asid(1);

        // 搴旇 TLB miss 鐒跺悗璧伴〉琛?
        let old_misses = mmu.tlb.stats.misses;
        assert_eq!(mmu.translate(0x100), Some(0x300));
        assert_eq!(mmu.tlb.stats.misses, old_misses + 1);
    }

    #[test]
    fn test_mmu_hit_rate() {
        let mut mmu = Mmu::new(4);
        mmu.current_asid = 0;
        mmu.add_mapping(0, 0x1, 0x10, 0x7);

        // 绗竴娆?miss
        mmu.translate(0x1);
        // 鍚庣画 9 娆?hit
        for _ in 0..9 {
            mmu.translate(0x1);
        }

        assert_eq!(mmu.tlb.stats.hits, 9);
        assert_eq!(mmu.tlb.stats.misses, 1);
        let rate = mmu.tlb.stats.hit_rate();
        assert!(
            (rate - 0.9).abs() < 1e-9,
            "hit rate should be 0.9, got {rate}"
        );
    }

    #[test]
    fn test_mmu_thrashing() {
        // TLB 鍙湁 2 涓Ы锛屼絾浜ゆ浛璁块棶 3 涓笉鍚岀殑椤?
        let mut mmu = Mmu::new(2);
        mmu.current_asid = 0;
        mmu.add_mapping(0, 0x1, 0x10, 0x7);
        mmu.add_mapping(0, 0x2, 0x20, 0x7);
        mmu.add_mapping(0, 0x3, 0x30, 0x7);

        // 璁块棶 1, 2, 3, 1, 2, 3 鈥?鐢变簬瀹归噺鍙湁 2锛屼細鎸佺画 miss锛坱hrashing锛?
        for vpn in [1, 2, 3, 1, 2, 3] {
            mmu.translate(vpn);
        }

        // 鍓嶄袱娆′竴瀹?miss锛堝喎鍚姩锛夛紝绗笁娆′篃 miss锛堟窐姹?vpn=1锛夛紝
        // 绗洓娆?vpn=1 琚窐姹颁簡鎵€浠?miss ... 鍏ㄩ儴 miss
        assert_eq!(mmu.tlb.stats.misses, 6);
        assert_eq!(mmu.tlb.stats.hits, 0);
    }
}
