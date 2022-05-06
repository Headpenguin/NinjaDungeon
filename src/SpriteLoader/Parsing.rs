use std::io::{self, Error, ErrorKind};

pub enum ParsedAnimation<'a> {
	Flip(usize),
	Standard(Vec<&'a str>, Vec<usize>),
}

pub fn parse<'a>(file: &'a str, names: &[&str]) -> io::Result<Vec<ParsedAnimation<'a>>> {
	let mut animations = vec![];
	for name in names {
		let mut iter = file.lines().skip_while(|l| !(l.ends_with(':') && l.contains(name))).skip(1).take_while(|l| !l.ends_with(';'));
		if let Some(s) = iter.next() {
			if let Some((_, s)) = s.split_once("&") {
				if let Some(p) = names.iter().position(|n| *n == s) {
					animations.push(ParsedAnimation::Flip(p))
				}
				else {
					return Err(Error::new(ErrorKind::InvalidData, format!("Reference to nonexistant animation \"{}\"", s)));
				}
			} //Fix bugs by processing current iteration in fallback case
			else {
				let mut paths = vec![];
				let mut positions = vec![];
				parseStandardLine(s, &mut paths, &mut positions)?;
				for line in iter {
					parseStandardLine(line, &mut paths, &mut positions)?;
				}
				animations.push(ParsedAnimation::Standard(paths, positions));
			}
		}
		else {
			return Err(Error::new(ErrorKind::UnexpectedEof, format!("Animation file did not contain animation {}", name)));
		}
	}
	Ok(animations)
}

#[inline(always)]
fn parseStandardLine<'a>(line: &'a str, paths: &mut Vec<&'a str>, positions: &mut Vec<usize>) -> io::Result<()> {
	if let (Some(begin), Some(end)) = (line.find('"'), line.rfind('"')) {	
		let path = &line[begin + 1..end];
		if let Some(p) = paths.iter().position(|p| *p == path) {
			positions.push(p);
		}
		else {
			paths.push(path);
			positions.push(positions.len());
		}
		Ok(())
	}
	else {
		Err(Error::new(ErrorKind::InvalidData, "Expected quotation marks surrounding all file names"))
	}
}

