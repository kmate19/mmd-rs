use std::io::Read;

use crate::pmx::{self, Header, Pmx};

pub(crate) struct Parser<R: Read, F: MMDFormat> {
    pub(crate) reader: R,
    globals: Option<F::Global>,
}

pub(crate) trait MMDFormat: Sized {
    type Global;
}

impl MMDFormat for Pmx {
    type Global = pmx::Globals;
}

impl<R: Read> Parser<R, Pmx> {
    pub(crate) fn new(reader: R) -> Self {
        Self {
            reader,
            globals: None,
        }
    }

    pub(crate) fn parse_header(&mut self) -> Result<pmx::Header, pmx::Error> {
        let r = &mut self.reader;

        // 4 bytes since there's a space after
        let mut tag = [0; 4];
        r.read_exact(&mut tag)?;
        if &tag[..3] != b"PMX" {
            Err(pmx::Error::InvalidTag)?
        }

        let mut ver = [0; 4];
        r.read_exact(&mut ver)?;
        let version = f32::from_le_bytes(ver);

        let globals = pmx::Globals::from_bytes(r)?;
        self.globals = Some(globals.clone());

        let name = self.parse()?;

        let comment = self.parse()?;

        Ok(Header {
            version,
            globals,
            name,
            comment,
        })
    }

    pub(crate) fn parse<P: PmxParseable>(&mut self) -> Result<P, P::Error> {
        let globals = self
            .globals
            .as_ref()
            .expect("Globals must be set before parsing, run parse_header first");

        // TODO(mate): just use typestate or something and remove this later
        //
        // SAFETY: globals field is private, trait impls can't mutate it, so this reference is valid for the duration of the parse call.
        // we need this so we can pass, self mutably, and we need that so we can parse types recursively
        // we could also just not pass the globals, and only pass parser, making globals accessible through parser.globals, but that means
        // that every impl needs to unwrap the option themselves, which is tedious.
        let globals_ref = unsafe { &*(globals as *const _) };

        P::parse(self, globals_ref)
    }
}

pub(crate) trait PmxParseable: Sized {
    type Error;

    fn parse<R: Read>(
        parser: &mut Parser<R, Pmx>,
        globals: &pmx::Globals,
    ) -> Result<Self, Self::Error>;
}
