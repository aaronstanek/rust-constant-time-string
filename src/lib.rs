#[derive(Debug, Clone, Copy)]
pub enum BytesPerCharacter {
    OneByte,
    TwoBytes,
    FourBytes,
}

fn bytes_to_encode_character(ch: char) -> BytesPerCharacter {
    if u32::from(ch) <= 0xFF {
        return BytesPerCharacter::OneByte;
    }
    if u32::from(ch) <= 0xFFFF {
        return BytesPerCharacter::TwoBytes;
    }
    return BytesPerCharacter::FourBytes;
}

#[derive(Debug, Clone, Default)]
enum ConstantTimeStringImpl {
    #[default]
    Empty,
    OneByte(Vec<u8>),
    TwoBytes(Vec<u16>),
    FourBytes(Vec<u32>),
}

impl ConstantTimeStringImpl {
    fn grow(self, minimum: BytesPerCharacter) -> Self {
        match self {
            ConstantTimeStringImpl::Empty => match minimum {
                BytesPerCharacter::OneByte => {
                    return ConstantTimeStringImpl::OneByte(Vec::new());
                }
                BytesPerCharacter::TwoBytes => {
                    return ConstantTimeStringImpl::TwoBytes(Vec::new());
                }
                BytesPerCharacter::FourBytes => {
                    return ConstantTimeStringImpl::FourBytes(Vec::new());
                }
            },
            ConstantTimeStringImpl::OneByte(ref old_content) => match minimum {
                BytesPerCharacter::OneByte => {
                    return self;
                }
                BytesPerCharacter::TwoBytes => {
                    let mut new_content: Vec<u16> = Vec::with_capacity(old_content.len());
                    for value in old_content.into_iter() {
                        new_content.push(*value as u16);
                    }
                    return ConstantTimeStringImpl::TwoBytes(new_content);
                }
                BytesPerCharacter::FourBytes => {
                    let mut new_content: Vec<u32> = Vec::with_capacity(old_content.len());
                    for value in old_content.into_iter() {
                        new_content.push(*value as u32);
                    }
                    return ConstantTimeStringImpl::FourBytes(new_content);
                }
            },
            ConstantTimeStringImpl::TwoBytes(ref old_content) => match minimum {
                BytesPerCharacter::OneByte | BytesPerCharacter::TwoBytes => {
                    return self;
                }
                BytesPerCharacter::FourBytes => {
                    let mut new_content: Vec<u32> = Vec::with_capacity(old_content.len());
                    for value in old_content.into_iter() {
                        new_content.push(*value as u32);
                    }
                    return ConstantTimeStringImpl::FourBytes(new_content);
                }
            },
            ConstantTimeStringImpl::FourBytes(_) => {
                return self;
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ConstantTimeString {
    content: ConstantTimeStringImpl,
}

impl ConstantTimeString {
    pub fn new() -> Self {
        Self {
            content: ConstantTimeStringImpl::Empty,
        }
    }
    fn push(&mut self, ch: char) {
        self.content = ConstantTimeStringImpl::grow(
            std::mem::take(&mut self.content),
            bytes_to_encode_character(ch),
        );
        match &mut self.content {
            ConstantTimeStringImpl::Empty => {}
            ConstantTimeStringImpl::OneByte(content) => {
                content.push(ch as u8);
            }
            ConstantTimeStringImpl::TwoBytes(content) => {
                content.push(ch as u16);
            }
            ConstantTimeStringImpl::FourBytes(content) => {
                content.push(ch as u32);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn string_character_width_helper(content: ConstantTimeStringImpl) -> u8 {
        match content {
            ConstantTimeStringImpl::Empty => 0,
            ConstantTimeStringImpl::OneByte(_) => 1,
            ConstantTimeStringImpl::TwoBytes(_) => 2,
            ConstantTimeStringImpl::FourBytes(_) => 4,
        }
    }

    #[test]
    fn empty_string_has_character_width_0() {
        let string = ConstantTimeString::new();
        assert_eq!(string_character_width_helper(string.content), 0);
    }

    #[test]
    fn ascii_character_has_character_width_1() {
        let mut string = ConstantTimeString::new();
        string.push('a');
        assert_eq!(string_character_width_helper(string.content), 1);
    }

    #[test]
    fn latin1_character_has_character_width_1() {
        let mut string = ConstantTimeString::new();
        string.push('ä');
        assert_eq!(string_character_width_helper(string.content), 1);
    }

    #[test]
    fn bmp_character_has_character_width_2() {
        let mut string = ConstantTimeString::new();
        string.push('人');
        assert_eq!(string_character_width_helper(string.content), 2);
    }

    #[test]
    fn non_bmp_character_has_character_width_4() {
        let mut string = ConstantTimeString::new();
        string.push('🙂');
        assert_eq!(string_character_width_helper(string.content), 4);
    }

    #[test]
    fn two_latin1_characters_has_character_width_1() {
        let mut string = ConstantTimeString::new();
        string.push('ä');
        string.push('ä');
        assert_eq!(string_character_width_helper(string.content), 1);
    }

    #[test]
    fn two_bmp_characters_has_character_width_2() {
        let mut string = ConstantTimeString::new();
        string.push('人');
        string.push('人');
        assert_eq!(string_character_width_helper(string.content), 2);
    }

    #[test]
    fn two_non_bmp_characters_has_character_width_4() {
        let mut string = ConstantTimeString::new();
        string.push('🙂');
        string.push('🙂');
        assert_eq!(string_character_width_helper(string.content), 4);
    }

    #[test]
    fn latin1_character_plus_bmp_character_has_character_width_2() {
        let mut string = ConstantTimeString::new();
        string.push('ä');
        string.push('人');
        assert_eq!(string_character_width_helper(string.content), 2);
    }

    #[test]
    fn latin1_character_plus_non_bmp_character_has_character_width_4() {
        let mut string = ConstantTimeString::new();
        string.push('ä');
        string.push('🙂');
        assert_eq!(string_character_width_helper(string.content), 4);
    }

    #[test]
    fn bmp_character_plus_latin1_character_has_character_width_2() {
        let mut string = ConstantTimeString::new();
        string.push('人');
        string.push('ä');
        assert_eq!(string_character_width_helper(string.content), 2);
    }

    #[test]
    fn bmp_character_plus_non_bmp_character_has_character_width_4() {
        let mut string = ConstantTimeString::new();
        string.push('人');
        string.push('🙂');
        assert_eq!(string_character_width_helper(string.content), 4);
    }

    #[test]
    fn non_bmp_character_plus_latin1_character_has_character_width_4() {
        let mut string = ConstantTimeString::new();
        string.push('🙂');
        string.push('ä');
        assert_eq!(string_character_width_helper(string.content), 4);
    }

    #[test]
    fn non_bmp_character_plus_bmp_character_has_character_width_4() {
        let mut string = ConstantTimeString::new();
        string.push('🙂');
        string.push('人');
        assert_eq!(string_character_width_helper(string.content), 4);
    }
}
