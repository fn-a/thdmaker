use std::io::BufRead;
use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;
use quick_xml::name::LocalName;
use super::error::{Error, Result};
use super::define::primitive::Color;
use super::define::material::*;

impl Texture2D {
    pub fn parse(elem: &BytesStart) -> Result<Self> {
        let mut id = 0u32;
        let mut path = String::new();
        let mut content_type = ContentType::default();
        let mut tile_style_u = TileStyle::Wrap;
        let mut tile_style_v = TileStyle::Wrap;
        let mut filter = Filter::Auto;

        for attr in elem.attributes().flatten() {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;

            match key {
                b"id" => id = value.parse()?,
                b"path" => path = value.to_string(),
                b"contenttype" => content_type = value.parse()?,
                b"tilestyleu" => tile_style_u = value.parse()?,
                b"tilestylev" => tile_style_v = value.parse()?,
                b"filter" => filter = value.parse()?,
                _ => {}
            }
        }

        Ok(Texture2D {
            id,
            path,
            content_type,
            tile_style_u,
            tile_style_v,
            filter,
        })
    }
}

impl ColorGroup {
    pub fn parse<R: BufRead>(reader: &mut Reader<R>, elem: &BytesStart) -> Result<Self> {
        let mut id = 0u32;
        let mut display_properties_id = None;

        for attr in elem.attributes().flatten() {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;

            match key {
                b"id" => id = value.parse()?,
                b"displaypropertiesid" => display_properties_id = Some(value.parse()?),
                _ => {}
            }
        }

        let mut color_group = ColorGroup {
            id,
            display_properties_id,
            colors: Vec::new(),
        };

        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) if e.local_name().as_ref() == b"color" => {
                    let color_elem = e;
                    let mut color = Color::default();

                    for attr in color_elem.attributes().flatten() {
                        let key = attr.key.as_ref();
                        let value = attr.unescape_value()?;

                        if key == b"color" {
                            color = value.parse()?;
                        }
                    }

                    color_group.colors.push(ColorType { color });
                }
                Ok(Event::End(ref e)) if e.local_name().as_ref() == b"colorgroup" => break,
                Ok(Event::Eof) => return Err(Error::UnexpectedEofIn("colorgroup".to_string())),
                Ok(_) => {}
                Err(e) => return Err(Error::Xml(e)),
            }
            buf.clear();
        }

        Ok(color_group)
    }
}

impl Texture2DGroup {
    pub fn parse<R: BufRead>(reader: &mut Reader<R>, elem: &BytesStart) -> Result<Self> {
        let mut id = 0u32;
        let mut tex_id = 0u32;
        let mut display_properties_id = None;

        for attr in elem.attributes().flatten() {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;

            match key {
                b"id" => id = value.parse()?,
                b"texid" => tex_id = value.parse()?,
                b"displaypropertiesid" => display_properties_id = Some(value.parse()?),
                _ => {}
            }
        }

        let mut texture_group = Texture2DGroup {
            id,
            tex_id,
            display_properties_id,
            tex_2_coords: Vec::new(),
        };

        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) if e.local_name().as_ref() == b"tex2coord" => {
                    let mut u = 0.0;
                    let mut v = 0.0;

                    for attr in e.attributes().flatten() {
                        let key = attr.key.as_ref();
                        let value = attr.unescape_value()?;

                        match key {
                            b"u" => u = value.parse()?,
                            b"v" => v = value.parse()?,
                            _ => {}
                        }
                    }

                    texture_group.tex_2_coords.push(Tex2Coord { u, v });
                }
                Ok(Event::End(ref e)) if e.local_name().as_ref() == b"texture2dgroup" => break,
                Ok(Event::Eof) => return Err(Error::UnexpectedEofIn("texture2dgroup".to_string())),
                Ok(_) => {}
                Err(e) => return Err(Error::Xml(e)),
            }
            buf.clear();
        }

        Ok(texture_group)
    }
}

impl CompositeMaterials {
    pub fn parse<R: BufRead>(reader: &mut Reader<R>, elem: &BytesStart) -> Result<Self> {
        let mut id = 0u32;
        let mut mat_id = 0u32;
        let mut mat_indices = Vec::new();
        let mut display_properties_id = None;

        for attr in elem.attributes().flatten() {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;

            match key {
                b"id" => id = value.parse()?,
                b"matid" => mat_id = value.parse()?,
                b"matindices" => mat_indices = value.split_whitespace()
                    .map(|v| v.parse::<u32>().map_err(|_| Error::InvalidAttribute {
                        name: "matindices".to_string(),
                        message: format!("invalid number: {}", v),
                    }))
                    .collect::<Result<Vec<_>>>()?,
                b"displaypropertiesid" => display_properties_id = Some(value.parse()?),
                _ => {}
            }
        }

        let mut composite_materials = CompositeMaterials {
            id,
            mat_id,
            mat_indices,
            display_properties_id,
            composites: Vec::new(),
        };

        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) if e.local_name().as_ref() == b"composite" => {
                    let mut values = Vec::new();

                    for attr in e.attributes().flatten() {
                        let key = attr.key.as_ref();
                        let value = attr.unescape_value()?;

                        if key == b"values" {
                            values = value.split_whitespace()
                                .map(|v| v.parse::<f64>().map_err(|_| Error::InvalidAttribute {
                                    name: "values".to_string(),
                                    message: format!("invalid number: {}", v),
                                }))
                                .collect::<Result<Vec<_>>>()?;
                        }
                    }

                    composite_materials.composites.push(Composite { values });
                }
                Ok(Event::End(ref e)) if e.local_name().as_ref() == b"compositematerials" => break,
                Ok(Event::Eof) => return Err(Error::UnexpectedEofIn("compositematerials".to_string())),
                Ok(_) => {}
                Err(e) => return Err(Error::Xml(e)),
            }
            buf.clear();
        }

        Ok(composite_materials)
    }
}

impl MultiProperties {
    pub fn parse<R: BufRead>(reader: &mut Reader<R>, elem: &BytesStart) -> Result<Self> {
        let mut id = 0u32;
        let mut p_ids = Vec::new();
        let mut blend_methods = Vec::new();

        for attr in elem.attributes().flatten() {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;

            match key {
                b"id" => id = value.parse()?,
                b"pids" => p_ids = value.split_whitespace()
                    .map(|v| v.parse::<u32>().map_err(|_| Error::InvalidAttribute {
                        name: "pids".to_string(),
                        message: format!("invalid number: {}", v),
                    }))
                    .collect::<Result<Vec<_>>>()?,
                b"blendmethods" => {
                    blend_methods = value.parse::<BlendMethods>()?.into();
                }
                _ => {}
            }
        }

        let mut multi_properties = MultiProperties {
            id,
            p_ids,
            blend_methods,
            multis: Vec::new(),
        };

        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) if e.local_name().as_ref() == b"multi" => {
                    let mut p_indices = Vec::new();

                    for attr in e.attributes().flatten() {
                        let key = attr.key.as_ref();
                        let value = attr.unescape_value()?;

                        if key == b"pindices" {
                            p_indices = value.split_whitespace()
                                .map(|v| v.parse::<u32>().map_err(|_| Error::InvalidAttribute {
                                    name: "pindices".to_string(),
                                    message: format!("invalid number: {}", v),
                                }))
                                .collect::<Result<Vec<_>>>()?;
                        }
                    }

                    multi_properties.multis.push(Multi { p_indices });
                }
                Ok(Event::End(ref e)) if e.local_name().as_ref() == b"multiproperties" => break,
                Ok(Event::Eof) => return Err(Error::UnexpectedEofIn("multiproperties".to_string())),
                Ok(_) => {}
                Err(e) => return Err(Error::Xml(e)),
            }
            buf.clear();
        }

        Ok(multi_properties)
    }
}

impl PBSpecularDisplayProperties {
    pub fn parse<R: BufRead>(reader: &mut Reader<R>, elem: &BytesStart) -> Result<Self> {
        let mut id = 0u32;

        for attr in elem.attributes().flatten() {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;

            if key == b"id" {
                id = value.parse()?;
            }
        }

        let mut pbspecular_props = PBSpecularDisplayProperties {
            id,
            pb_speculars: Vec::new(),
        };

        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) if e.local_name().as_ref() == b"pbspecular" => {
                    let pbspecular = PBSpecular::parse(e)?;
                    pbspecular_props.pb_speculars.push(pbspecular);
                }
                Ok(Event::End(ref e)) if e.local_name().as_ref() == b"pbspeculardisplayproperties" => break,
                Ok(Event::Eof) => return Err(Error::UnexpectedEofIn("pbspeculardisplayproperties".to_string())),
                Ok(_) => {}
                Err(e) => return Err(Error::Xml(e)),
            }
            buf.clear();
        }

        Ok(pbspecular_props)
    }
}

impl PBSpecular {
    fn parse(elem: &BytesStart) -> Result<Self> {
        let mut name = String::new();
        let mut specular_color = Color::rgb(0x38, 0x38, 0x38);
        let mut glossiness = 0.0;

        for attr in elem.attributes().flatten() {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;

            match key {
                b"name" => name = value.to_string(),
                b"specularcolor" => specular_color = value.parse()?,
                b"glossiness" => glossiness = value.parse()?,
                _ => {}
            }
        }

        Ok(PBSpecular {
            name,
            specular_color,
            glossiness,
        })
    }
}

impl PBMetallicDisplayProperties {
    pub fn parse<R: BufRead>(reader: &mut Reader<R>, elem: &BytesStart) -> Result<Self> {
        let mut id = 0u32;

        for attr in elem.attributes().flatten() {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;

            if key == b"id" {
                id = value.parse()?;
            }
        }

        let mut pbmetallic_props = PBMetallicDisplayProperties {
            id,
            pb_metallics: Vec::new(),
        };

        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) if e.local_name().as_ref() == b"pbmetallic" => {
                    let pbmetallic = PBMetallic::parse(e)?;
                    pbmetallic_props.pb_metallics.push(pbmetallic);
                }
                Ok(Event::End(ref e)) if e.local_name().as_ref() == b"pbmetallicdisplayproperties" => break,
                Ok(Event::Eof) => return Err(Error::UnexpectedEofIn("pbmetallicdisplayproperties".to_string())),
                Ok(_) => {}
                Err(e) => return Err(Error::Xml(e)),
            }
            buf.clear();
        }

        Ok(pbmetallic_props)
    }
}

impl PBMetallic {
    fn parse(elem: &BytesStart) -> Result<Self> {
        let mut name = String::new();
        let mut metallicness = 0.0;
        let mut roughness = 1.0;

        for attr in elem.attributes().flatten() {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;

            match key {
                b"name" => name = value.to_string(),
                b"metallicness" => metallicness = value.parse()?,
                b"roughness" => roughness = value.parse()?,
                _ => {}
            }
        }

        Ok(PBMetallic {
            name,
            metallicness,
            roughness,
        })
    }
}

impl PBSpecularTextureDisplayProperties {
    pub fn parse(elem: &BytesStart) -> Result<Self> {
        let mut id = 0u32;
        let mut name = String::new();
        let mut specular_texture_id = 0u32;
        let mut glossiness_texture_id = 0u32;
        let mut diffuse_factor = Color::WHITE;
        let mut specular_factor = Color::WHITE;
        let mut glossiness_factor = 1.0;

        for attr in elem.attributes().flatten() {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;

            match key {
                b"id" => id = value.parse()?,
                b"name" => name = value.to_string(),
                b"speculartextureid" => specular_texture_id = value.parse()?,
                b"glossinesstextureid" => glossiness_texture_id = value.parse()?,
                b"diffusefactor" => diffuse_factor = value.parse()?,
                b"specularfactor" => specular_factor = value.parse()?,
                b"glossinessfactor" => glossiness_factor = value.parse()?,
                _ => {}
            }
        }

        Ok(PBSpecularTextureDisplayProperties {
            id,
            name,
            specular_texture_id,
            glossiness_texture_id,
            diffuse_factor,
            specular_factor,
            glossiness_factor,
        })
    }
}

impl PBMetallicTextureDisplayProperties {
    pub fn parse(elem: &BytesStart) -> Result<Self> {
        let mut id = 0u32;
        let mut name = String::new();
        let mut metallic_texture_id = 0u32;
        let mut roughness_texture_id = 0u32;
        let mut base_color_factor = Color::WHITE;
        let mut metallic_factor = 1.0;
        let mut roughness_factor = 1.0;

        for attr in elem.attributes().flatten() {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;

            match key {
                b"id" => id = value.parse()?,
                b"name" => name = value.to_string(),
                b"metallictextureid" => metallic_texture_id = value.parse()?,
                b"roughnesstextureid" => roughness_texture_id = value.parse()?,
                b"basecolorfactor" => base_color_factor = value.parse()?,
                b"metallicfactor" => metallic_factor = value.parse()?,
                b"roughnessfactor" => roughness_factor = value.parse()?,
                _ => {}
            }
        }

        Ok(PBMetallicTextureDisplayProperties {
            id,
            name,
            metallic_texture_id,
            roughness_texture_id,
            base_color_factor,
            metallic_factor,
            roughness_factor,
        })
    }
}

impl TranslucentDisplayProperties {
    pub fn parse<R: BufRead>(reader: &mut Reader<R>, elem: &BytesStart) -> Result<Self> {
        let mut id = 0u32;

        for attr in elem.attributes().flatten() {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;

            if key == b"id" {
                id = value.parse()?;
            }
        }

        let mut translucent_display_props = TranslucentDisplayProperties {
            id,
            translucents: Vec::new(),
        };

        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) if e.local_name().as_ref() == b"translucent" => {
                    let translucent = Translucent::parse(e)?;
                    translucent_display_props.translucents.push(translucent);
                }
                Ok(Event::End(ref e)) if e.local_name().as_ref() == b"translucentdisplayproperties" => break,
                Ok(Event::Eof) => return Err(Error::UnexpectedEofIn("translucentdisplayproperties".to_string())),
                Ok(_) => {}
                Err(e) => return Err(Error::Xml(e)),
            }
            buf.clear();
        }

        Ok(translucent_display_props)
    }
}

impl Translucent {
    fn parse(elem: &BytesStart) -> Result<Self> {
        let mut name = String::new();
        let mut attenuation = [0.0, 0.0, 0.0];
        let mut refractive_index = [1.0, 1.0, 1.0];
        let mut roughness = 0.0;

        for attr in elem.attributes().flatten() {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;

            match key {
                b"name" => name = value.to_string(),
                b"attenuation" => {
                    let values = value.split_whitespace()
                        .map(|v| v.parse::<f64>().map_err(|_| Error::InvalidAttribute {
                            name: "attenuation".to_string(),
                            message: format!("invalid number: {}", v),
                        }))
                        .collect::<Result<Vec<_>>>()?;
                    if values.len() >= 3 {
                        attenuation = [values[0], values[1], values[2]];
                    }
                }
                b"refractiveindex" => {
                    let values = value.split_whitespace()
                        .map(|v| v.parse::<f64>().map_err(|_| Error::InvalidAttribute {
                            name: "refractiveindex".to_string(),
                            message: format!("invalid number: {}", v),
                        }))
                        .collect::<Result<Vec<_>>>()?;
                    if values.len() >= 3 {
                        refractive_index = [values[0], values[1], values[2]];
                    }
                }
                b"roughness" => roughness = value.parse()?,
                _ => {}
            }
        }

        Ok(Translucent {
            name,
            attenuation,
            refractive_index,
            roughness,
        })
    }
}

impl MaterialResources {
    pub fn parse<R: BufRead>(&mut self, name: &LocalName, reader: &mut Reader<R>, elem: &BytesStart) -> Result<Option<u32>> {
        let mut next_id = None;
        match name.as_ref() {
            b"texture2d" => {
                let texture2d = Texture2D::parse(elem)?;
                next_id = Some(texture2d.id);
                self.add_texture_2d(texture2d);
            }
            b"colorgroup" => {
                let color_group = ColorGroup::parse(reader, elem)?;
                next_id = Some(color_group.id);
                self.add_color_group(color_group);
            }
            b"texture2dgroup" => {
                let texture_group = Texture2DGroup::parse(reader, elem)?;
                next_id = Some(texture_group.id);
                self.add_texture_2d_group(texture_group);
            }
            b"compositematerials" => {
                let composite = CompositeMaterials::parse(reader, elem)?;
                next_id = Some(composite.id);
                self.add_composite_materials(composite);
            }
            b"multiproperties" => {
                let multi_props = MultiProperties::parse(reader, elem)?;
                next_id = Some(multi_props.id);
                self.add_multi_properties(multi_props);
            }
            b"pbspeculardisplayproperties" => {
                let props = PBSpecularDisplayProperties::parse(reader, elem)?;
                next_id = Some(props.id);
                self.add_pb_specular_display_properties(props);
            }
            b"pbmetallicdisplayproperties" => {
                let props = PBMetallicDisplayProperties::parse(reader, elem)?;
                next_id = Some(props.id);
                self.add_pb_metallic_display_properties(props);
            }
            b"pbspeculartexturedisplayproperties" => {
                let props = PBSpecularTextureDisplayProperties::parse(elem)?;
                next_id = Some(props.id);
                self.add_pb_specular_texture_display_properties(props);
            }
            b"pbmetallictexturedisplayproperties" => {
                let props = PBMetallicTextureDisplayProperties::parse(elem)?;
                next_id = Some(props.id);
                self.add_pb_metallic_texture_display_properties(props);
            }
            b"translucentdisplayproperties" => {
                let props = TranslucentDisplayProperties::parse(reader, elem)?;
                next_id = Some(props.id);
                self.add_translucent_display_properties(props);
            }
            _ => {}
        }

        Ok(next_id)
    }
}