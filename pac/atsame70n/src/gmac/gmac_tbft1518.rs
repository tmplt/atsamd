#[doc = "Register `GMAC_TBFT1518` reader"]
pub struct R(crate::R<GMAC_TBFT1518_SPEC>);
impl core::ops::Deref for R {
    type Target = crate::R<GMAC_TBFT1518_SPEC>;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl From<crate::R<GMAC_TBFT1518_SPEC>> for R {
    #[inline(always)]
    fn from(reader: crate::R<GMAC_TBFT1518_SPEC>) -> Self {
        R(reader)
    }
}
#[doc = "Field `NFTX` reader - 1024 to 1518 Byte Frames Transmitted without Error"]
pub struct NFTX_R(crate::FieldReader<u32, u32>);
impl NFTX_R {
    #[inline(always)]
    pub(crate) fn new(bits: u32) -> Self {
        NFTX_R(crate::FieldReader::new(bits))
    }
}
impl core::ops::Deref for NFTX_R {
    type Target = crate::FieldReader<u32, u32>;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl R {
    #[doc = "Bits 0:31 - 1024 to 1518 Byte Frames Transmitted without Error"]
    #[inline(always)]
    pub fn nftx(&self) -> NFTX_R {
        NFTX_R::new(self.bits as u32)
    }
}
#[doc = "1024 to 1518 Byte Frames Transmitted Register\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [gmac_tbft1518](index.html) module"]
pub struct GMAC_TBFT1518_SPEC;
impl crate::RegisterSpec for GMAC_TBFT1518_SPEC {
    type Ux = u32;
}
#[doc = "`read()` method returns [gmac_tbft1518::R](R) reader structure"]
impl crate::Readable for GMAC_TBFT1518_SPEC {
    type Reader = R;
}
#[doc = "`reset()` method sets GMAC_TBFT1518 to value 0"]
impl crate::Resettable for GMAC_TBFT1518_SPEC {
    #[inline(always)]
    fn reset_value() -> Self::Ux {
        0
    }
}
