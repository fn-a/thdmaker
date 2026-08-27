use std::collections::HashMap;
use std::io::BufRead;
use std::str::FromStr;
use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;
use quick_xml::name::LocalName;
use super::error::{Error, Result};
use super::define::volumetric::*;
use super::define::primitive::*;

impl Function {
    /// Parse a function element.
    fn parse<R: BufRead>(name: &LocalName, reader: &mut Reader<R>, elem: &BytesStart, event: &Event) -> Result<Self> {
        let mut id = 0u32;
        let mut display_name = None;
        let mut function_type = None;

        for attr in elem.attributes().flatten() {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;

            match key {
                b"id" => id = value.parse()?,
                b"displayname" => display_name = Some(value.to_string()),
                _ => {}
            }
        }

        match name.as_ref() {
            b"functionfromimage3d" => {
                
                let function_image = FunctionFromImage3D::parse(reader, elem, event)?;
                function_type = Some(FunctionType::FunctionFromImage3D(function_image));
            }
            b"privateextensionfunction" => {
                let function_private = PrivateExtensionFunction::parse(reader, elem, event)?;
                function_type = Some(FunctionType::PrivateExtensionFunction(function_private));
            }
            _ => {}
        }

        let function_type = function_type.ok_or_else(|| {
            Error::InvalidStructure(format!("function type not matched: {}", String::from_utf8_lossy(name.as_ref())))
        })?;

        Ok(Self { id, display_name, r#type: function_type })
    }
}

impl FunctionFromImage3D {
    /// Parse an empty functionfromimage3d element.
    fn parse<R: BufRead>(reader: &mut Reader<R>, elem: &BytesStart, event: &Event) -> Result<Self> {
        let mut image_3d_id = 0u32;
        let mut value_offset = 0.0;
        let mut value_scale = 1.0;
        let mut filter = Filter::Linear;
        let mut tile_style_u = TileStyle::Wrap;
        let mut tile_style_v = TileStyle::Wrap;
        let mut tile_style_w = TileStyle::Wrap;

        for attr in elem.attributes().flatten() {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;

            match key {
                b"image3did" => image_3d_id = value.parse()?,
                b"valueoffset" => value_offset = value.parse()?,
                b"valuescale" => value_scale = value.parse()?,
                b"filter" => filter = value.parse()?,
                b"tilestyleu" => tile_style_u = value.parse()?,
                b"tilestylev" => tile_style_v = value.parse()?,
                b"tilestylew" => tile_style_w = value.parse()?,
                _ => {}
            }
        }

        if let Event::Start(_) = event {
            let mut buf = Vec::new();
            loop {
                match reader.read_event_into(&mut buf) {
                    Ok(Event::End(_)) => break,
                    Ok(Event::Eof) => return Err(Error::UnexpectedEofIn("functionfromimage3d".to_string())),
                    Ok(_) => {}
                    Err(e) => return Err(Error::Xml(e)),
                }
                buf.clear();
            }
        }

        Ok(Self {
            image_3d_id,
            value_offset,
            value_scale,
            filter,
            tile_style_u,
            tile_style_v,
            tile_style_w,
        })
    }
}

impl PrivateExtensionFunction {
    /// Parse a private extension function element.
    fn parse<R: BufRead>(reader: &mut Reader<R>, elem: &BytesStart, event: &Event) -> Result<Self> {
        let mut attrs = HashMap::new();
        let mut content = None;

        for attr in elem.attributes().flatten() {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;
            attrs.insert(String::from_utf8_lossy(key).to_string(), value.to_string());
        }

        if let Event::Start(_) = event {
            // Skip to end
            let mut txt = String::new();
            let mut buf = Vec::new();
            loop {
                match reader.read_event_into(&mut buf) {
                    Ok(Event::End(e)) if e.name().as_ref() == b"privateextensionfunction" => break,
                    Ok(Event::Eof) => return Err(Error::UnexpectedEofIn("privateextensionfunction".to_string())),
                    Ok(_) => {}
                    Err(e) => return Err(Error::Xml(e)),
                }
                txt += String::from_utf8_lossy(&buf).as_ref();
                buf.clear();
            }
            content = Some(txt);
        }

        Ok(Self { attributes: attrs, content })
    }
}

impl Image3D {
    /// Parse an image3d element.
    fn parse<R: BufRead>(reader: &mut Reader<R>, elem: &BytesStart) -> Result<Self> {
        let mut id = 0u32;
        let mut name = None;
        let mut content = None;

        for attr in elem.attributes().flatten() {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;

            match key {
                b"id" => id = value.parse()?,
                b"name" => name = Some(value.to_string()),
                _ => {}
            }
        }

        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    match e.local_name().as_ref() {
                        b"imagestack" => {
                            let stack = ImageStack::parse(reader, e)?;
                            content = Some(Image3DContent::ImageStack(stack));
                        }
                        _ => {}
                    }
                }
                Ok(Event::End(ref e)) if e.name().as_ref() == b"image3d" => break,
                Ok(Event::Eof) => return Err(Error::UnexpectedEofIn("image3d".to_string())),
                Ok(_) => {}
                Err(e) => return Err(Error::Xml(e)),
            }
            buf.clear();
        }

        let content = content.ok_or_else(|| {
            Error::InvalidStructure("image3d must have an imagestack child".to_string())
        })?;

        Ok(Self { id, name, content })
    }
}

impl ImageStack {
    /// Parse an imagestack element.
    fn parse<R: BufRead>(reader: &mut Reader<R>, elem: &BytesStart) -> Result<Self> {
        let mut row_count = 0u32;
        let mut column_count = 0u32;
        let mut sheet_count = 0u32;
        let mut sheets = Vec::new();


        for attr in elem.attributes().flatten() {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;

            match key {
                b"rowcount" => row_count = value.parse()?,
                b"columncount" => column_count = value.parse()?,
                b"sheetcount" => sheet_count = value.parse()?,
                _ => {}
            }
        }

        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Empty(ref e)) => {
                    match e.local_name().as_ref()  {
                        b"imagesheet" => {
                            let sheet = ImageSheet::parse(e)?;
                            sheets.push(sheet);
                        }
                        _ => {}
                    }
                }
                Ok(Event::End(ref e)) if e.name().as_ref() == b"imagestack" => break,
                Ok(Event::Eof) => return Err(Error::UnexpectedEofIn("imagestack".to_string())),
                Ok(_) => {}
                Err(e) => return Err(Error::Xml(e)),
            }
            buf.clear();
        }

        Ok(Self {
            row_count,
            column_count,
            sheet_count,
            image_sheets: sheets,
        })
    }
}

impl ImageSheet {
    /// Parse an imagesheet element.
    fn parse(elem: &BytesStart) -> Result<Self> {
        let mut path = String::new();

        for attr in elem.attributes().flatten() {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;

            if key == b"path" {
                path = value.to_string();
            }
        }

        if path.is_empty() {
            return Err(Error::InvalidAttribute {
                name: "path".to_string(),
                message: "imagesheet must have a path attribute".to_string(),
            });
        }

        Ok(Self { path })
    }
}

impl VolumeData {
    /// Parse a volumedata element.
    fn parse<R: BufRead>(reader: &mut Reader<R>, elem: &BytesStart) -> Result<Self> {
        let mut id = 0u32;
        let mut color = None;
        let mut composite = None;
        let mut properties = Vec::new();

        for attr in elem.attributes().flatten() {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;

            if key == b"id" {
                id = value.parse()?;
            }
        }

        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    match e.local_name().as_ref() {
                        b"color" => color = Some(Colour::parse(reader, e)?),
                        b"composite" => composite = Some(Composite::parse(reader, e)?),
                        b"property" => properties.push(Property::parse(reader, e)?),
                        _ => {}
                    }
                }
                Ok(Event::Empty(ref e)) => {
                    match e.local_name().as_ref() {
                        b"color" => color = Some(Colour::attributes(e)?),
                        b"property" => properties.push(Property::attributes(e)?),
                        _ => {}
                    }
                }
                Ok(Event::End(ref e)) if e.name().as_ref() == b"volumedata" => break,
                Ok(Event::Eof) => return Err(Error::UnexpectedEofIn("volumedata".to_string())),
                Ok(_) => {}
                Err(e) => return Err(Error::Xml(e)),
            }
            buf.clear();
        }

        Ok(Self { id, color, composite, properties })
    }
}

impl Colour {
    /// Parse a color element.
    fn parse<R: BufRead>(reader: &mut Reader<R>, elem: &BytesStart) -> Result<Self> {
        let color = Self::attributes(elem)?;

        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::End(ref e)) if e.name().as_ref() == b"color" => break,
                Ok(Event::Eof) => return Err(Error::UnexpectedEofIn("color".to_string())),
                Ok(_) => {}
                Err(e) => return Err(Error::Xml(e)),
            }
            buf.clear();
        }

        Ok(color)
    }

    /// Parse a color element from attributes.
    fn attributes(elem: &BytesStart) -> Result<Self> {
        let mut function_id = 0u32;
        let mut transform = None;
        let mut channel = String::new();
        let mut min_feature_size = 0.0;
        let mut fallback_value = 0.0;

        for attr in elem.attributes().flatten() {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;

            match key {
                b"functionid" => function_id = value.parse()?,
                b"transform" => transform = Some(Matrix3D::from_str(&value)?),
                b"channel" => channel = value.to_string(),
                b"minfeaturesize" => min_feature_size = value.parse()?,
                b"fallbackvalue" => fallback_value = value.parse()?,
                _ => {}
            }
        }

        Ok(Self {
            function_id,
            transform,
            channel,
            min_feature_size,
            fallback_value,
        })
    }
}

impl Composite {
    /// Parse a composite element.
    fn parse<R: BufRead>(reader: &mut Reader<R>, elem: &BytesStart) -> Result<Self> {
        let mut base_material_id = 0u32;
        let mut mappings = Vec::new();

        for attr in elem.attributes().flatten() {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;

            if key == b"basematerialid" {
                base_material_id = value.parse()?;
            }
        }

        let mut buf = Vec::new();

        // Read material mappings
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    match e.local_name().as_ref() {
                        b"materialmapping" => mappings.push(MaterialMapping::parse(reader, e)?),
                        _ => {}
                    }
                }
                Ok(Event::Empty(ref e)) => {
                    match e.local_name().as_ref() {
                        b"materialmapping" => mappings.push(MaterialMapping::attributes(e)?),
                        _ => {}
                    }
                }
                Ok(Event::End(ref e)) if e.name().as_ref() == b"composite" => break,
                Ok(Event::Eof) => return Err(Error::UnexpectedEofIn("composite".to_string())),
                Ok(_) => {}
                Err(e) => return Err(Error::Xml(e)),
            }
            buf.clear();
        }

        Ok(Self { base_material_id, material_mappings: mappings })
    }
}

impl MaterialMapping {
    /// Parse a materialmapping element.
    fn parse<R: BufRead>(reader: &mut Reader<R>, elem: &BytesStart) -> Result<Self> {
        let mapping = Self::attributes(elem)?;

        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::End(ref e)) if e.name().as_ref() == b"materialmapping" => break,
                Ok(Event::Eof) => return Err(Error::UnexpectedEofIn("materialmapping".to_string())),
                Ok(_) => {}
                Err(e) => return Err(Error::Xml(e)),
            }
            buf.clear();
        }

        Ok(mapping)
    }

    /// Parse an empty materialmapping element.
    fn attributes(elem: &BytesStart) -> Result<Self> {
        let mut function_id = 0u32;
        let mut transform = None;
        let mut channel = String::new();
        let mut min_feature_size = 0.0;
        let mut fallback_value = 0.0;

        for attr in elem.attributes().flatten() {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;

            match key {
                b"functionid" => function_id = value.parse()?,
                b"transform" => transform = Some(Matrix3D::from_str(&value)?),
                b"channel" => channel = value.to_string(),
                b"minfeaturesize" => min_feature_size = value.parse()?,
                b"fallbackvalue" => fallback_value = value.parse()?,
                _ => {}
            }
        }

        Ok(Self {
            function_id,
            transform,
            channel,
            min_feature_size,
            fallback_value,
        })
    }
}

impl Property {
    /// Parse a property element.
    fn parse<R: BufRead>(reader: &mut Reader<R>, elem: &BytesStart) -> Result<Self> {
        let property = Self::attributes(elem)?;

        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::End(ref e)) if e.name().as_ref() == b"property" => break,
                Ok(Event::Eof) => return Err(Error::UnexpectedEofIn("property".to_string())),
                Ok(_) => {}
                Err(e) => return Err(Error::Xml(e)),
            }
            buf.clear();
        }

        Ok(property)
    }

    /// Parse an empty property element.
    fn attributes(elem: &BytesStart) -> Result<Self> {
        let mut function_id = 0u32;
        let mut transform = None;
        let mut channel = String::new();
        let mut name = String::new();
        let mut required = false;
        let mut min_feature_size = 0.0;
        let mut fallback_value = 0.0;

        for attr in elem.attributes().flatten() {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;

            match key {
                b"functionid" => function_id = value.parse()?,
                b"channel" => channel = value.to_string(),
                b"transform" => transform = Some(Matrix3D::from_str(&value)?),
                b"name" => name = value.to_string(),
                b"required" => required = value == "1" || value == "true",
                b"minfeaturesize" => min_feature_size = value.parse()?,
                b"fallbackvalue" => fallback_value = value.parse()?,
                _ => {}
            }
        }

        Ok(Self {
            function_id,
            transform,
            channel,
            name,
            required,
            min_feature_size,
            fallback_value,
        })
    }
}

impl LevelSet {
    /// Parse a levelset element.
    pub fn parse<R: BufRead>(reader: &mut Reader<R>, elem: &BytesStart, event: &Event) -> Result<Self> {
        let mut function_id = 0u32;
        let mut channel = String::new();
        let mut transform = None;
        let mut min_feature_size = 0.0;
        let mut mesh_id = 0u32;
        let mut mesh_bbox_only = false;
        let mut fallback_value = 0.0;
        let mut volume_id = None;


        for attr in elem.attributes().flatten() {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;

            match key {
                b"functionid" => function_id = value.parse()?,
                b"channel" => channel = value.to_string(),
                b"transform" => transform = Some(Matrix3D::from_str(&value)?),
                b"minfeaturesize" => min_feature_size = value.parse()?,
                b"meshid" => mesh_id = value.parse()?,
                b"meshbboxonly" => mesh_bbox_only = value == "1" || value == "true",
                b"fallbackvalue" => fallback_value = value.parse()?,
                b"volumeid" => volume_id = Some(value.parse()?),
                _ => {}
            }
        }

        if let Event::Start(_) = event {
            let mut buf = Vec::new();
            loop {
                match reader.read_event_into(&mut buf) {
                    Ok(Event::End(ref e)) if e.name().as_ref() == b"levelset" => break,
                    Ok(Event::Eof) => return Err(Error::UnexpectedEofIn("levelset".to_string())),
                    Ok(_) => {}
                    Err(e) => return Err(Error::Xml(e)),
                }
                buf.clear();
            }
        }

        Ok(Self {
            function_id,
            channel,
            transform,
            min_feature_size,
            mesh_id,
            mesh_bbox_only,
            fallback_value,
            volume_id,
        })
    }
}

impl VolumetricResources {
    /// Parse volumetric resources from XML.
    pub fn parse<R: BufRead>(&mut self, name: &LocalName, reader: &mut Reader<R>, elem: &BytesStart, event: &Event) -> Result<Option<u32>> {
        let mut next_id = None;

        match name.as_ref() {
            b"functionfromimage3d" | b"privateextensionfunction" => {
                let function = Function::parse(name, reader, elem, event)?;
                next_id = Some(function.id);
                self.functions.push(function);
            }
            b"image3d" => {
                let image3d = Image3D::parse(reader, elem)?;
                next_id = Some(image3d.id);
                self.image_3ds.push(image3d);
            }
            b"volumedata" => {
                let volumedata = VolumeData::parse(reader, elem)?;
                next_id = Some(volumedata.id);
                self.volume_datas.push(volumedata);
            }
            _ => {} // Skip unknown elements
        }

        Ok(next_id)
    }
}
