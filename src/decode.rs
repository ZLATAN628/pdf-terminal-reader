use anyhow::bail;
use chardetng::EncodingDetector;
use encoding_rs::Encoding;

pub fn decode_str_to_utf8(bytes: &[u8]) -> anyhow::Result<String> {
    let mut detector = EncodingDetector::new();
    detector.feed(bytes, true);
    let fed = detector.guess(None, true).name();
    let encoding = Encoding::for_label(fed.as_bytes());
    if let Some(encode) = encoding {
        let (cow, _, has_error) = encode.decode(bytes);
        if !has_error {
            return Ok(cow.into());
        }
    }
    bail!("decode error")
}