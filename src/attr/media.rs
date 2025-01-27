use super::ipp_sys_predefined_map::IppSysPredefinedMap;
use std::sync::OnceLock;
use winprint::ticket::{PageMediaSize, PredefinedMediaName};

const COMMON_MEDIA_PAIR: &[(&str, PredefinedMediaName)] = &[
    ("iso_a0_841x1189mm", PredefinedMediaName::ISOA0),
    ("iso_a1_594x841mm", PredefinedMediaName::ISOA1),
    ("iso_a10_26x37mm", PredefinedMediaName::ISOA10),
    ("iso_a2_420x594mm", PredefinedMediaName::ISOA2),
    ("iso_a3_297x420mm", PredefinedMediaName::ISOA3),
    ("iso_a3-extra_322x445mm", PredefinedMediaName::ISOA3Extra),
    ("iso_a4_210x297mm", PredefinedMediaName::ISOA4),
    (
        "iso_a4-extra_235.5x322.3mm",
        PredefinedMediaName::ISOA4Extra,
    ),
    ("iso_a5_148x210mm", PredefinedMediaName::ISOA5),
    ("iso_a5-extra_174x235mm", PredefinedMediaName::ISOA5Extra),
    ("iso_a6_105x148mm", PredefinedMediaName::ISOA6),
    ("iso_a7_74x105mm", PredefinedMediaName::ISOA7),
    ("iso_a8_52x74mm", PredefinedMediaName::ISOA8),
    ("iso_a9_37x52mm", PredefinedMediaName::ISOA9),
    ("iso_b0_1000x1414mm", PredefinedMediaName::ISOB0),
    ("iso_b1_707x1000mm", PredefinedMediaName::ISOB1),
    ("iso_b10_31x44mm", PredefinedMediaName::ISOB10),
    ("iso_b2_500x707mm", PredefinedMediaName::ISOB2),
    ("iso_b3_353x500mm", PredefinedMediaName::ISOB3),
    ("iso_b4_250x353mm", PredefinedMediaName::ISOB4),
    ("iso_b5-extra_201x276mm", PredefinedMediaName::ISOB5Extra),
    ("iso_b5_176x250mm", PredefinedMediaName::ISOB5Envelope),
    ("iso_b7_88x125mm", PredefinedMediaName::ISOB7),
    ("iso_b8_62x88mm", PredefinedMediaName::ISOB8),
    ("iso_b9_44x62mm", PredefinedMediaName::ISOB9),
    ("iso_c0_917x1297mm", PredefinedMediaName::ISOC0),
    ("iso_c1_648x917mm", PredefinedMediaName::ISOC1),
    ("iso_c10_28x40mm", PredefinedMediaName::ISOC10),
    ("iso_c2_458x648mm", PredefinedMediaName::ISOC2),
    ("iso_c3_324x458mm", PredefinedMediaName::ISOC3Envelope),
    ("iso_c4_229x324mm", PredefinedMediaName::ISOC4Envelope),
    ("iso_c5_162x229mm", PredefinedMediaName::ISOC5Envelope),
    ("iso_c6_114x162mm", PredefinedMediaName::ISOC6Envelope),
    ("iso_c6c5_114x229mm", PredefinedMediaName::ISOC6C5Envelope),
    ("iso_c7_81x114mm", PredefinedMediaName::ISOC7),
    ("iso_c8_57x81mm", PredefinedMediaName::ISOC8),
    ("iso_c9_40x57mm", PredefinedMediaName::ISOC9),
    ("iso_dl_110x220mm", PredefinedMediaName::ISODLEnvelope),
    ("iso_sra2_450x640mm", PredefinedMediaName::ISOSRA3),
    ("jis_b0_1030x1456mm", PredefinedMediaName::JISB0),
    ("jis_b1_728x1030mm", PredefinedMediaName::JISB1),
    ("jis_b10_32x45mm", PredefinedMediaName::JISB10),
    ("jis_b2_515x728mm", PredefinedMediaName::JISB2),
    ("jis_b3_364x515mm", PredefinedMediaName::JISB3),
    ("jis_b4_257x364mm", PredefinedMediaName::JISB4),
    ("jis_b5_182x257mm", PredefinedMediaName::JISB5),
    ("jis_b6_128x182mm", PredefinedMediaName::JISB6),
    ("jis_b7_91x128mm", PredefinedMediaName::JISB7),
    ("jis_b8_64x91mm", PredefinedMediaName::JISB8),
    ("jis_b9_45x64mm", PredefinedMediaName::JISB9),
    (
        "jpn_chou3_120x235mm",
        PredefinedMediaName::JapanChou3Envelope,
    ),
    (
        "jpn_chou4_90x205mm",
        PredefinedMediaName::JapanChou4Envelope,
    ),
    (
        "jpn_hagaki_100x148mm",
        PredefinedMediaName::JapanHagakiPostcard,
    ),
    (
        "jpn_kaku2_240x332mm",
        PredefinedMediaName::JapanKaku2Envelope,
    ),
    (
        "jpn_kaku3_216x277mm",
        PredefinedMediaName::JapanKaku3Envelope,
    ),
    ("jpn_you4_105x235mm", PredefinedMediaName::JapanYou4Envelope),
    ("na_10x11_10x11in", PredefinedMediaName::NorthAmerica10x11),
    ("na_10x14_10x14in", PredefinedMediaName::NorthAmerica10x14),
    ("na_5x7_5x7in", PredefinedMediaName::NorthAmerica5x7),
    ("na_9x11_9x11in", PredefinedMediaName::NorthAmerica9x11),
    (
        "na_arch-a_9x12in",
        PredefinedMediaName::NorthAmericaArchitectureASheet,
    ),
    (
        "na_arch-b_12x18in",
        PredefinedMediaName::NorthAmericaArchitectureBSheet,
    ),
    (
        "na_arch-c_18x24in",
        PredefinedMediaName::NorthAmericaArchitectureCSheet,
    ),
    (
        "na_arch-d_24x36in",
        PredefinedMediaName::NorthAmericaArchitectureDSheet,
    ),
    (
        "na_arch-e_36x48in",
        PredefinedMediaName::NorthAmericaArchitectureESheet,
    ),
    ("na_c_17x22in", PredefinedMediaName::NorthAmericaCSheet),
    ("na_d_22x34in", PredefinedMediaName::NorthAmericaDSheet),
    ("na_e_34x44in", PredefinedMediaName::NorthAmericaESheet),
    (
        "na_executive_7.25x10.5in",
        PredefinedMediaName::NorthAmericaExecutive,
    ),
    ("na_ledger_11x17in", PredefinedMediaName::NorthAmerica11x17),
    (
        "na_legal-extra_9.5x15in",
        PredefinedMediaName::NorthAmericaLegalExtra,
    ),
    ("na_legal_8.5x14in", PredefinedMediaName::NorthAmericaLegal),
    (
        "na_letter-extra_9.5x12in",
        PredefinedMediaName::NorthAmericaLetterExtra,
    ),
    (
        "na_letter-plus_8.5x12.69in",
        PredefinedMediaName::NorthAmericaLetterPlus,
    ),
    (
        "na_letter_8.5x11in",
        PredefinedMediaName::NorthAmericaLetter,
    ),
    (
        "na_monarch_3.875x7.5in",
        PredefinedMediaName::NorthAmericaMonarchEnvelope,
    ),
    (
        "na_number-10_4.125x9.5in",
        PredefinedMediaName::NorthAmericaNumber10Envelope,
    ),
    (
        "na_number-11_4.5x10.375in",
        PredefinedMediaName::NorthAmericaNumber11Envelope,
    ),
    (
        "na_number-12_4.75x11in",
        PredefinedMediaName::NorthAmericaNumber12Envelope,
    ),
    (
        "na_number-14_5x11.5in",
        PredefinedMediaName::NorthAmericaNumber14Envelope,
    ),
    (
        "na_number-9_3.875x8.875in",
        PredefinedMediaName::NorthAmericaNumber9Envelope,
    ),
    (
        "na_personal_3.625x6.5in",
        PredefinedMediaName::NorthAmericaPersonalEnvelope,
    ),
    (
        "na_quarto_8.5x10.83in",
        PredefinedMediaName::NorthAmericaQuarto,
    ),
    (
        "na_super-a_8.94x14in",
        PredefinedMediaName::NorthAmericaSuperA,
    ),
    (
        "na_super-b_13x19in",
        PredefinedMediaName::NorthAmericaSuperB,
    ),
    ("prc_10_324x458mm", PredefinedMediaName::PRC10Envelope),
    ("prc_16k_146x215mm", PredefinedMediaName::PRC16K),
    ("prc_1_102x165mm", PredefinedMediaName::PRC1Envelope),
    ("prc_2_102x176mm", PredefinedMediaName::PRC2Envelope),
    ("prc_32k_97x151mm", PredefinedMediaName::PRC32K),
    ("prc_3_125x176mm", PredefinedMediaName::PRC3Envelope),
    ("prc_4_110x208mm", PredefinedMediaName::PRC4Envelope),
    ("prc_5_110x220mm", PredefinedMediaName::PRC5Envelope),
    ("prc_6_120x320mm", PredefinedMediaName::PRC6Envelope),
    ("prc_7_160x230mm", PredefinedMediaName::PRC7Envelope),
    ("prc_8_120x309mm", PredefinedMediaName::PRC8Envelope),
];

struct CommonMediaMap {}

impl IppSysPredefinedMap for CommonMediaMap {
    type IppKey = &'static str;
    type SysPredefined = PredefinedMediaName;
    type SysOptionPack = PageMediaSize;

    fn bimap() -> &'static bimap::BiHashMap<Self::IppKey, Self::SysPredefined> {
        static BIMAP: OnceLock<bimap::BiHashMap<&str, PredefinedMediaName>> = OnceLock::new();
        BIMAP.get_or_init(|| bimap::BiHashMap::from_iter(COMMON_MEDIA_PAIR.iter().copied()))
    }
}

pub fn all_supported_media(packs: impl Iterator<Item = PageMediaSize>) -> Vec<String> {
    CommonMediaMap::all_supported_by_win(packs)
        .iter()
        .map(|x| x.to_string())
        .collect()
}

pub fn get_media_by_ipp(
    packs: impl Iterator<Item = PageMediaSize>,
    ipp_key: &str,
) -> Option<PageMediaSize> {
    CommonMediaMap::find_by_ipp(packs, ipp_key)
}
