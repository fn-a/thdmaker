use std::str::FromStr;
use std::io::BufRead;
use quick_xml::events::{BytesStart, Event};
use quick_xml::escape::unescape;
use quick_xml::Reader;
use super::error::{Error, Result};
use super::define::model::*;
use super::define::primitive::*;
use super::define::booleanoperations::BooleanShape;
use super::define::displacement::DisplacementMesh;
use super::define::production::{ProductionItem, ProductionComponent, ProductionBuild, ProductionObject, Alternatives};
use super::define::slice::SliceObject;
use super::define::volumetric::LevelSet;

impl Model {
    pub fn parse<R: BufRead>(reader: R) -> Result<Self> {
        let mut xml_reader = Reader::from_reader(reader);
        xml_reader.config_mut().trim_text(true);

        let mut model = Self::new();
        let mut buf = Vec::new();

        loop {
            match xml_reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    match e.name().as_ref() {
                        b"model" => {
                            model.attributes(e)?;
                        }
                        b"metadata" => {
                            let meta = Metadata::parse(&mut xml_reader, e)?;
                            model.metadatas.push(meta);
                        }
                        b"resources" => {
                            model.resources.parse(&mut xml_reader)?;
                        }
                        b"build" => {
                            model.build.parse(&mut xml_reader, e)?;
                        }
                        _ => {} // Ignore unknown elements
                    }
                }
                Ok(Event::Eof) => break,
                Ok(_) => {} // Skip other events
                Err(e) => return Err(Error::Xml(e)),
            }
            buf.clear();
        }

        Ok(model)
    }

    fn attributes(&mut self, elem: &BytesStart) -> Result<()> {
        for attr in elem.attributes().flatten() {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;

            match key {
                b"unit" => self.unit = value.parse()?,
                b"xml:lang" => self.language = Some(value.to_string()),
                b"requiredextensions" => {
                    self.required_extensions = value.split_whitespace()
                        .map(|s| s.to_string())
                        .collect();
                }
                b"recommendedextensions" => {
                    self.recommended_extensions = value.split_whitespace()
                        .map(|s| s.to_string())
                        .collect();
                }
                _ => {} // Ignore unknown attributes
            }
        }
        Ok(())
    }
}

impl Metadata {
    fn parse<R: BufRead>(reader: &mut Reader<R>, elem: &BytesStart) -> Result<Self> {
        let mut meta = Self::default();
    
        for attr in elem.attributes().flatten() {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;
    
            match key {
                b"name" => meta.name = value.to_string(),
                b"preserve" => meta.preserve = value == "1" || value == "true",
                b"type" => meta.r#type = Some(value.to_string()),
                _ => {}
            }
        }
    
        // Read content
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Text(e)) => {
                    let content = e.decode()?;
                    meta.value = unescape(&content)?.to_string();
                }
                Ok(Event::End(ref e)) if e.name().as_ref() == b"metadata" => break,
                Ok(Event::Eof) => return Err(Error::UnexpectedEofIn("metadata".to_string())),
                Ok(_) => {}
                Err(e) => return Err(Error::Xml(e)),
            }
            buf.clear();
        }
    
        Ok(meta)
    }
}

impl Resources {
    fn parse<R: BufRead>(&mut self, reader: &mut Reader<R>,) -> Result<()> {
        let mut buf = Vec::new();
    
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(ref event @ Event::Start(ref e)) => {
                    let (name, prefix) = e.name().decompose();
                    if let Some(prefix) = prefix {
                        match prefix.as_ref() {
                            b"d" => {
                                // Displacement extension parts
                                let disp = self.displacements.get_or_insert_with(Default::default);
                                if let Some(id) = disp.parse(&name, reader, e)? {
                                    self.set_next_id(id);
                                }
                            }
                            b"m" => {
                                // Material extension parts
                                let mat = self.materials.get_or_insert_with(Default::default);
                                if let Some(id) = mat.parse(&name, reader, e)? {
                                    self.set_next_id(id);
                                }
                            }
                            b"s" => {
                                // Slice extension parts
                                let slice = self.slices.get_or_insert_with(Default::default);
                                if let Some(id) = slice.parse(&name, reader, e)? {
                                    self.set_next_id(id);
                                }
                            }
                            b"v" => {
                                // Volumetric extension parts
                                let vol = self.volumetric.get_or_insert_with(Default::default);
                                if let Some(id) = vol.parse(&name, reader, e, event)? {
                                    self.set_next_id(id);
                                }
                            }
                            b"i" => {
                                // Implicit extension parts
                                let imp = self.implicit.get_or_insert_with(Default::default);
                                if let Some(id) = imp.parse(&name, reader, e)? {
                                    self.set_next_id(id);
                                }
                            }
                            _ => {} // Unknown resource extension part
                        }
                    } else {
                        match e.name().as_ref() {
                            b"basematerials" => {
                                let materials = BaseMaterials::parse(reader, e)?;
                                self.set_next_id(materials.id);
                                self.base_materials.push(materials);
                            }
                            b"object" => {
                                let object = Object::parse(reader, e)?;
                                self.set_next_id(object.id);
                                self.objects.push(object);
                            }
                            _ => {} // Unknown resource part
                        }
                    }
                }
                Ok(ref event @ Event::Empty(ref e)) => {
                    let (name, prefix) = e.name().decompose();
                    if let Some(prefix) = prefix {
                        match prefix.as_ref() {
                            b"v" => {
                                // Volumetric extension parts
                                let vol = self.volumetric.get_or_insert_with(Default::default);
                                if let Some(id) = vol.parse(&name, reader, e, event)? {
                                    self.set_next_id(id);
                                }
                            }
                            _ => {} // Unknown resource extension part
                        }
                    } else {
                        match e.name().as_ref() {
                            b"object" => {
                                // Empty object (no mesh or components) - unusual but possible
                                let object = Object::attributes(e)?;
                                self.set_next_id(object.id);
                                self.objects.push(object);
                            }
                            _ => {} // Unknown resource part
                        }
                    }
                }
                Ok(Event::End(ref e)) if e.name().as_ref() == b"resources" => break,
                Ok(Event::Eof) => return Err(Error::UnexpectedEofIn("resources".to_string())),
                Ok(_) => {}
                Err(e) => return Err(Error::Xml(e)),
            }
            buf.clear();
        }
    
        Ok(())
    }
}

impl BaseMaterials {
    fn parse<R: BufRead>(reader: &mut Reader<R>, elem: &BytesStart) -> Result<Self> {
        let mut id = 0u32;
        let mut props_id = None;
        
        for attr in elem.attributes().flatten() {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;
    
            if key == b"id" {
                id = value.parse()?;
            }

            // Material extension display properties ID
            if key == b"displaypropertiesid" {
                props_id = Some(value.parse()?);
            }
        }
    
        let mut materials = Self::new(id);
        materials.display_properties_id = props_id;
        let mut buf = Vec::new();
    
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Empty(ref e)) if e.name().as_ref() == b"base" => {
                    let material = BaseMaterial::parse(e)?;
                    materials.materials.push(material);
                }
                Ok(Event::End(ref e)) if e.name().as_ref() == b"basematerials" => break,
                Ok(Event::Eof) => return Err(Error::UnexpectedEofIn("basematerials".to_string())),
                Ok(_) => {}
                Err(e) => return Err(Error::Xml(e)),
            }
            buf.clear();
        }
    
        Ok(materials)
    }
}

impl BaseMaterial {
    fn parse(elem: &BytesStart) -> Result<Self> {
        let mut name = String::new();
        let mut display_color = Color::WHITE;
    
        for attr in elem.attributes().flatten() {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;
    
            match key {
                b"name" => name = value.to_string(),
                b"displaycolor" => display_color = Color::from_str(&value)?,
                _ => {}
            }
        }
    
        Ok(Self { name, display_color })
    }
}

impl Object {
    fn parse<R: BufRead>(reader: &mut Reader<R>, elem: &BytesStart) -> Result<Self> {
        let mut object = Self::attributes(elem)?;
        let mut buf = Vec::new();
    
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(ref event @ Event::Start(ref e)) => {
                    let (name, prefix) = e.name().decompose();
                    if let Some(prefix) = prefix {
                        match prefix.as_ref() {
                            b"bo" => {
                                match name.as_ref() {
                                     b"booleanshape" => {
                                        let boolean_shape = BooleanShape::parse(reader, e)?;
                                        object.content = ObjectContent::BooleanShape(boolean_shape);
                                    }
                                    _ => {} // Unknown boolean shape element
                                }
                            }
                            b"d" => {
                                match name.as_ref() {
                                    b"displacementmesh" => {
                                        let displacement_mesh = DisplacementMesh::parse(reader)?;
                                        object.content = ObjectContent::DisplacementMesh(displacement_mesh);
                                    }
                                    _ => {} // Unknown displacement mesh element
                                }
                            }
                            b"pa" => {
                                match name.as_ref() {
                                    b"alternatives" => {
                                        let alternatives = Alternatives::parse(reader)?;
                                        object.content = ObjectContent::Alternatives(alternatives);
                                    }
                                    _ => {} // Unknown alternatives element
                                }
                            }
                            b"v" => {
                                match name.as_ref() {
                                    b"levelset" => {
                                        let levelset = LevelSet::parse(reader, e, event)?;
                                        object.content = ObjectContent::LevelSet(levelset);
                                    }
                                    _ => {} // Unknown volumetric element
                                }
                            }
                            _ => {} // Unknown object extension part
                        }
                    } else {
                        match name.as_ref() {
                            b"metadatagroup" => {
                                object.metadata_group = MetadataGroup::parse(reader)?;
                            }
                            b"mesh" => {
                                let mesh = Mesh::parse(reader, e)?;
                                object.content = ObjectContent::Mesh(mesh);
                            }
                            b"components" => {
                                let components = Components::parse(reader)?;
                                object.content = ObjectContent::Components(components);
                            }
                            _ => {} // Unknown object part
                        }
                    }
                }
                Ok(ref event @ Event::Empty(ref e)) => {
                    let (name, prefix) = e.name().decompose();
                    if let Some(prefix) = prefix {
                        match prefix.as_ref() {
                            b"v" => {
                                match name.as_ref() {
                                    b"levelset" => {
                                        let levelset = LevelSet::parse(reader, e, event)?;
                                        object.content = ObjectContent::LevelSet(levelset);
                                    }
                                    _ => {} // Unknown volumetric element
                                }
                            }
                            _ => {} // Unknown object extension part
                        }
                    }
                }
                Ok(Event::End(ref e)) if e.name().as_ref() == b"object" => break,
                Ok(Event::Eof) => return Err(Error::UnexpectedEofIn("object".to_string())),
                Ok(_) => {}
                Err(e) => return Err(Error::Xml(e)),
            }
            buf.clear();
        }
    
        Ok(object)
    }

    fn attributes(elem: &BytesStart) -> Result<Self> {
        let mut id = 0u32;
        let mut object_type = ObjectType::Model;
        let mut thumbnail = None;
        let mut part_number = None;
        let mut name = None;
        let mut pid = None;
        let mut pindex = None;
        let mut did = None;
        let mut production = None;
        let mut slice = None;

        let mut prod_attrs = Vec::new();
        let mut slice_attrs = Vec::new();

        for attr in elem.attributes().flatten() {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;

            match key {
                b"id" => id = value.parse()?,
                b"type" => object_type = value.parse()?,
                b"thumbnail" => thumbnail = Some(value.to_string()),
                b"partnumber" => part_number = Some(value.to_string()),
                b"name" => name = Some(value.to_string()),
                b"pid" => pid = Some(value.parse()?),
                b"pindex" => pindex = Some(value.parse()?),
                b"did" => did = Some(value.parse()?),
                _ => {
                    if let Some(prefix) = attr.key.prefix() {
                        match prefix.as_ref() {
                            b"p" | b"pa" => {
                                prod_attrs.push(attr);
                            }
                            b"s" => {
                                slice_attrs.push(attr);
                            }
                            _ => {} // Unknown object extension part
                        }
                    }
                }
            }
        }

        if !prod_attrs.is_empty() {
            production = Some(ProductionObject::parse(&prod_attrs)?);
        }

        if !slice_attrs.is_empty() {
            slice = Some(SliceObject::parse(&slice_attrs)?);
        }

        Ok(Self {
            id,
            r#type: object_type,
            thumbnail,
            part_number,
            name,
            pid,
            pindex,
            did,
            production,
            slice,
            metadata_group: MetadataGroup::new(),
            content: ObjectContent::Mesh(Mesh::new()),
        })
    }
}

impl MetadataGroup {
    fn parse<R: BufRead>(reader: &mut Reader<R>) -> Result<Self> {
        let mut metadatas = Vec::new();
        let mut buf = Vec::new();
    
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) if e.name().as_ref() == b"metadata" => {
                    let meta = Metadata::parse(reader, e)?;
                    metadatas.push(meta);
                }
                Ok(Event::End(ref e)) if e.name().as_ref() == b"metadatagroup" => break,
                Ok(Event::Eof) => return Err(Error::UnexpectedEofIn("metadatagroup".to_string())),
                Ok(_) => {}
                Err(e) => return Err(Error::Xml(e)),
            }
            buf.clear();
        }
    
        Ok(Self { metadatas })
    }
}

impl Components {
    fn parse<R: BufRead>(reader: &mut Reader<R>) -> Result<Self> {
        let mut components = Vec::new();
        let mut buf = Vec::new();
    
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Empty(ref e)) if e.name().as_ref() == b"component" => {
                    let component = Component::parse(e)?;
                    components.push(component);
                }
                Ok(Event::End(ref e)) if e.name().as_ref() == b"components" => break,
                Ok(Event::Eof) => return Err(Error::UnexpectedEofIn("components".to_string())),
                Ok(_) => {}
                Err(e) => return Err(Error::Xml(e)),
            }
            buf.clear();
        }
    
        Ok(Self { components })
    }
}

impl Component {
    fn parse(elem: &BytesStart) -> Result<Self> {
        let mut object_id = 0u32;
        let mut transform = None;
        let mut production = None;

        let mut prod_attrs = Vec::new();
    
        for attr in elem.attributes().flatten() {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;
    
            match key {
                b"objectid" => object_id = value.parse()?,
                b"transform" => transform = Some(Matrix3D::from_str(&value)?),
                _ => {
                    if let Some(prefix) = attr.key.prefix() {
                        match prefix.as_ref() {
                            b"p" => {
                                prod_attrs.push(attr);
                            }
                            _ => {} // Unknown object extension part
                        }
                    }
                }
            }
        }
    
        if !prod_attrs.is_empty() {
            production = Some(ProductionComponent::parse(&prod_attrs)?);
        }
    
        Ok(Self { object_id, transform, production })
    }
}

impl Build {
    fn parse<R: BufRead>(&mut self, reader: &mut Reader<R>, elem: &BytesStart) -> Result<()> {
        self.attributes(elem)?;
        let mut buf = Vec::new();
    
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) if e.name().as_ref() == b"item" => {
                    let item = Item::parse(reader, e)?;
                    self.items.push(item);
                }
                Ok(Event::Empty(ref e)) if e.name().as_ref() == b"item" => {
                    let item = Item::attributes(e)?;
                    self.items.push(item);
                }
                Ok(Event::End(ref e)) if e.name().as_ref() == b"build" => break,
                Ok(Event::Eof) => return Err(Error::UnexpectedEofIn("build".to_string())),
                Ok(_) => {}
                Err(e) => return Err(Error::Xml(e)),
            }
            buf.clear();
        }
    
        Ok(())
    }

    fn attributes(&mut self, elem: &BytesStart) -> Result<()> {
        let mut prod_attrs = Vec::new();

        for attr in elem.attributes().flatten() {
            if let Some(prefix) = attr.key.prefix() {
                match prefix.as_ref() {
                    b"p" => {
                        prod_attrs.push(attr);
                    }
                    _ => {} // Unknown object extension part
                }
            }
        }

        if !prod_attrs.is_empty() {
            self.production = Some(ProductionBuild::parse(&prod_attrs)?);
        }

        Ok(())
    }
}

impl Item {
    fn parse<R: BufRead>(reader: &mut Reader<R>, elem: &BytesStart) -> Result<Self> {
        let mut item = Self::attributes(elem)?;
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) if e.name().as_ref() == b"metadatagroup" => {
                    item.metadata_group = MetadataGroup::parse(reader)?;
                }
                Ok(Event::End(ref e)) if e.name().as_ref() == b"item" => break,
                Ok(Event::Eof) => return Err(Error::UnexpectedEofIn("item".to_string())),
                Ok(_) => {}
                Err(e) => return Err(Error::Xml(e)),
            }
            buf.clear();
        }

        Ok(item)
    }

    fn attributes(elem: &BytesStart) -> Result<Self> {
        let mut object_id = 0u32;
        let mut transform = None;
        let mut part_number = None;
        let mut production = None;

        let mut prod_attrs = Vec::new();
    
        for attr in elem.attributes().flatten() {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;
    
            match key {
                b"objectid" => object_id = value.parse()?,
                b"transform" => transform = Some(Matrix3D::from_str(&value)?),
                b"partnumber" => part_number = Some(value.to_string()),
                _ => {
                    if let Some(prefix) = attr.key.prefix() {
                        match prefix.as_ref() {
                            b"p" => {
                                prod_attrs.push(attr);
                            }
                            _ => {} // Unknown object extension part
                        }
                    }
                }
            }
        }

        if !prod_attrs.is_empty() {
            production = Some(ProductionItem::parse(&prod_attrs)?);
        }
    
        Ok(Self {
            object_id,
            transform,
            part_number,
            production,
            metadata_group: MetadataGroup::new(),
        })
    }
}