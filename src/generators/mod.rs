// Helianto -- static website generator
// Copyright © 2015-2016 Mickaël RAYBAUD-ROIG
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.


use std::rc::Rc;
use super::Result;
use super::document::{Document, DocumentMetadata};

mod index;
pub use self::index::IndexGenerator;



pub trait Generator {
    fn new() -> Self where Self: Sized;
    fn generate(&self, docs: &[Rc<DocumentMetadata>]) -> Result<Vec<Rc<Document>>>;
}

