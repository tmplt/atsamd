#[doc = "Register `GMAC_MFT` reader"]
pub struct R(crate::R<GMAC_MFT_SPEC>);
impl core::ops::Deref for R {
    type Target = crate::R<GMAC_MFT_SPEC>;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl From<crate::R<GMAC_MFT_SPEC>> for R {
    #[inline(always)]
    fn from(reader: crate::R<GMAC_MFT_SPEC>) -> Self {
        R(reader)
    }
}
#[doc = "Field `MFTX` reader - Multicast Frames Transmitted without Error"]
pub struct MFTX_R(crate::FieldReader<u32, u32>);
impl MFTX_R {
    #[inline(always)]
    pub(crate) fn new(bits: u32) -> Self {
        MFTX_R(crate::FieldReader::new(bits))
    }
}
impl core::ops::Deref for MFTX_R {
    type Target = crate::FieldReader<u32, u32>;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl R {
    #[doc = "Bits 0:31 - Multicast Frames Transmitted without Error"]
    #[inline(always)]
    pub fn mftx(&self) -> MFTX_R {
        MFTX_R::new(self.bits as u32)
    }
}
#[doc = "Multicast Frames Transmitted Register\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [gmac_mft](index.html) module"]
pub struct GMAC_MFT_SPEC;
impl crate::RegisterSpec for GMAC_MFT_SPEC {
    type Ux = u32;
}
#[doc = "`read()` method returns [gmac_mft::R](R) reader structure"]
impl crate::Readable for GMAC_MFT_SPEC {
    type Reader = R;
}
#[doc = "`reset()` method sets GMAC_MFT to value 0"]
impl crate::Resettable for GMAC_MFT_SPEC {
    #[inline(always)]
    fn reset_value() -> Self::Ux {
        0
    }
}
