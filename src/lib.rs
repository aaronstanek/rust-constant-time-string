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

    #[test]
    fn empty_string_has_character_width_0() {
        let string = ConstantTimeString::new();
        matches!(string.content, ConstantTimeStringImpl::Empty);
    }

    #[test]
    fn ascii_character_has_character_width_1() {
        let mut string = ConstantTimeString::new();
        string.push('a');
        matches!(string.content, ConstantTimeStringImpl::OneByte { .. });
    }

    #[test]
    fn latin1_character_has_character_width_1() {
        let mut string = ConstantTimeString::new();
        string.push('ä');
        matches!(string.content, ConstantTimeStringImpl::OneByte { .. });
    }

    #[test]
    fn bmp_character_has_character_width_2() {
        let mut string = ConstantTimeString::new();
        string.push('人');
        matches!(string.content, ConstantTimeStringImpl::TwoBytes { .. });
    }

    #[test]
    fn non_bmp_character_has_character_width_4() {
        let mut string = ConstantTimeString::new();
        string.push('🙂');
        matches!(string.content, ConstantTimeStringImpl::FourBytes { .. });
    }
}
