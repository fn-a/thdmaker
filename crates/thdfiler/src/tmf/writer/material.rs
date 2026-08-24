use std::io::Write;
use quick_xml::events::{BytesEnd, BytesStart, Event};
use quick_xml::Writer;
use super::error::Result;
use super::define::package::Namespace;
use super::define::primitive::Color;
use super::define::material::*;

impl Texture2D {
    pub fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut elem = BytesStart::new("m:texture2d");
        elem.push_attribute(("xmlns:m", Namespace::MATERIALS_NS));
        elem.push_attribute(("id", self.id.to_string().as_str()));
        elem.push_attribute(("path", self.path.as_str()));
        elem.push_attribute(("contenttype", self.content_type.to_string().as_str()));

        if self.tile_style_u != TileStyle::Wrap {
            elem.push_attribute(("tilestyleu", self.tile_style_u.to_string().as_str()));
        }

        if self.tile_style_v != TileStyle::Wrap {
            elem.push_attribute(("tilestylev", self.tile_style_v.to_string().as_str()));
        }

        if self.filter != Filter::Auto {
            elem.push_attribute(("filter", self.filter.to_string().as_str()));
        }

        writer.write_event(Event::Empty(elem))?;
        Ok(())
    }
}

impl ColorGroup {
    pub fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut elem = BytesStart::new("m:colorgroup");
        elem.push_attribute(("xmlns:m", Namespace::MATERIALS_NS));
        elem.push_attribute(("id", self.id.to_string().as_str()));

        if let Some(display_properties_id) = self.display_properties_id {
            elem.push_attribute(("displaypropertiesid", display_properties_id.to_string().as_str()));
        }

        writer.write_event(Event::Start(elem))?;

        for color_def in &self.colors {
            let mut color_elem = BytesStart::new("m:color");
            color_elem.push_attribute(("color", color_def.color.to_string().as_str()));
            writer.write_event(Event::Empty(color_elem))?;
        }

        writer.write_event(Event::End(BytesEnd::new("m:colorgroup")))?;
        Ok(())
    }
}

impl Texture2DGroup {
    pub fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut elem = BytesStart::new("m:texture2dgroup");
        elem.push_attribute(("xmlns:m", Namespace::MATERIALS_NS));
        elem.push_attribute(("id", self.id.to_string().as_str()));
        elem.push_attribute(("texid", self.tex_id.to_string().as_str()));

        if let Some(display_properties_id) = self.display_properties_id {
            elem.push_attribute(("displaypropertiesid", display_properties_id.to_string().as_str()));
        }

        writer.write_event(Event::Start(elem))?;

        for tex2_coord in &self.tex_2_coords {
            let mut coord_elem = BytesStart::new("m:tex2coord");
            coord_elem.push_attribute(("u", tex2_coord.u.to_string().as_str()));
            coord_elem.push_attribute(("v", tex2_coord.v.to_string().as_str()));
            writer.write_event(Event::Empty(coord_elem))?;
        }

        writer.write_event(Event::End(BytesEnd::new("m:texture2dgroup")))?;
        Ok(())
    }
}

impl CompositeMaterials {
    pub fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut elem = BytesStart::new("m:compositematerials");
        elem.push_attribute(("xmlns:m", Namespace::MATERIALS_NS));
        elem.push_attribute(("id", self.id.to_string().as_str()));
        elem.push_attribute(("matid", self.mat_id.to_string().as_str()));
        let indices = self.mat_indices.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(" ");
        elem.push_attribute(("matindices", indices.as_str()));

        if let Some(display_properties_id) = self.display_properties_id {
            elem.push_attribute(("displaypropertiesid", display_properties_id.to_string().as_str()));
        }

        writer.write_event(Event::Start(elem))?;

        for composite in &self.composites {
            let mut composite_elem = BytesStart::new("m:composite");
            let values = composite.values.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(" ");
            composite_elem.push_attribute(("values", values.as_str()));
            writer.write_event(Event::Empty(composite_elem))?;
        }

        writer.write_event(Event::End(BytesEnd::new("m:compositematerials")))?;
        Ok(())
    }
}

impl MultiProperties {
    pub fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut elem = BytesStart::new("m:multiproperties");
        elem.push_attribute(("xmlns:m", Namespace::MATERIALS_NS));
        elem.push_attribute(("id", self.id.to_string().as_str()));
        let ids = self.p_ids.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(" ");
        elem.push_attribute(("pids", ids.as_str()));

        if !self.blend_methods.is_empty() {
            let blend_methods_str = self.blend_methods.iter()
                .map(|m| m.to_string())
                .collect::<Vec<_>>()
                .join(" ");
            elem.push_attribute(("blendmethods", blend_methods_str.as_str()));
        }

        writer.write_event(Event::Start(elem))?;

        for multi in &self.multis {
            let mut multi_elem = BytesStart::new("m:multi");
            let indices = multi.p_indices.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(" ");
            multi_elem.push_attribute(("pindices", indices.as_str()));
            writer.write_event(Event::Empty(multi_elem))?;
        }

        writer.write_event(Event::End(BytesEnd::new("m:multiproperties")))?;
        Ok(())
    }
}

impl PBSpecularDisplayProperties {
    pub fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut elem = BytesStart::new("m:pbspeculardisplayproperties");
        elem.push_attribute(("xmlns:m", Namespace::MATERIALS_NS));
        elem.push_attribute(("id", self.id.to_string().as_str()));

        writer.write_event(Event::Start(elem))?;

        for pb_specular in &self.pb_speculars {
            pb_specular.write(writer)?;
        }

        writer.write_event(Event::End(BytesEnd::new("m:pbspeculardisplayproperties")))?;
        Ok(())
    }
}

impl PBSpecular {
    pub fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut elem = BytesStart::new("m:pbspecular");
        elem.push_attribute(("name", self.name.as_str()));

        if self.specular_color != Color::rgb(0x38, 0x38, 0x38) {
            elem.push_attribute(("specularcolor", self.specular_color.to_string().as_str()));
        }

        if self.glossiness != 0.0 {
            elem.push_attribute(("glossiness", self.glossiness.to_string().as_str()));
        }

        writer.write_event(Event::Empty(elem))?;
        Ok(())
    }
}

impl PBMetallicDisplayProperties {
    pub fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut elem = BytesStart::new("m:pbmetallicdisplayproperties");
        elem.push_attribute(("xmlns:m", Namespace::MATERIALS_NS));
        elem.push_attribute(("id", self.id.to_string().as_str()));

        writer.write_event(Event::Start(elem))?;

        for pb_metallic in &self.pb_metallics {
            pb_metallic.write(writer)?;
        }

        writer.write_event(Event::End(BytesEnd::new("m:pbmetallicdisplayproperties")))?;
        Ok(())
    }
}

impl PBMetallic {
    pub fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut elem = BytesStart::new("m:pbmetallic");
        elem.push_attribute(("name", self.name.as_str()));

        if self.metallicness != 0.0 {
            elem.push_attribute(("metallicness", self.metallicness.to_string().as_str()));
        }

        if self.roughness != 1.0 {
            elem.push_attribute(("roughness", self.roughness.to_string().as_str()));
        }

        writer.write_event(Event::Empty(elem))?;
        Ok(())
    }
}

impl PBSpecularTextureDisplayProperties {
    pub fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut elem = BytesStart::new("m:pbspeculartexturedisplayproperties");
        elem.push_attribute(("xmlns:m", Namespace::MATERIALS_NS));
        elem.push_attribute(("id", self.id.to_string().as_str()));
        elem.push_attribute(("name", self.name.as_str()));
        elem.push_attribute(("speculartextureid", self.specular_texture_id.to_string().as_str()));
        elem.push_attribute(("glossinesstextureid", self.glossiness_texture_id.to_string().as_str()));

        if self.diffuse_factor != Color::WHITE {
            elem.push_attribute(("diffusefactor", self.diffuse_factor.to_string().as_str()));
        }

        if self.specular_factor != Color::WHITE {
            elem.push_attribute(("specularfactor", self.specular_factor.to_string().as_str()));
        }

        if self.glossiness_factor != 1.0 {
            elem.push_attribute(("glossinessfactor", self.glossiness_factor.to_string().as_str()));
        }

        writer.write_event(Event::Empty(elem))?;
        Ok(())
    }
}

impl PBMetallicTextureDisplayProperties {
    pub fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut elem = BytesStart::new("m:pbmetallictexturedisplayproperties");
        elem.push_attribute(("xmlns:m", Namespace::MATERIALS_NS));
        elem.push_attribute(("id", self.id.to_string().as_str()));
        elem.push_attribute(("name", self.name.as_str()));
        elem.push_attribute(("metallictextureid", self.metallic_texture_id.to_string().as_str()));
        elem.push_attribute(("roughnesstextureid", self.roughness_texture_id.to_string().as_str()));

        if self.base_color_factor != Color::WHITE {
            elem.push_attribute(("basecolorfactor", self.base_color_factor.to_string().as_str()));
        }

        if self.metallic_factor != 1.0 {
            elem.push_attribute(("metallicfactor", self.metallic_factor.to_string().as_str()));
        }

        if self.roughness_factor != 1.0 {
            elem.push_attribute(("roughnessfactor", self.roughness_factor.to_string().as_str()));
        }

        writer.write_event(Event::Empty(elem))?;
        Ok(())
    }
}

impl TranslucentDisplayProperties {
    pub fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut elem = BytesStart::new("m:translucentdisplayproperties");
        elem.push_attribute(("xmlns:m", Namespace::MATERIALS_NS));
        elem.push_attribute(("id", self.id.to_string().as_str()));

        writer.write_event(Event::Start(elem))?;

        for translucent in &self.translucents {
            translucent.write(writer)?;
        }

        writer.write_event(Event::End(BytesEnd::new("m:translucentdisplayproperties")))?;
        Ok(())
    }
}

impl Translucent {
    pub fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut elem = BytesStart::new("m:translucent");
        elem.push_attribute(("name", self.name.as_str()));
        let attenuation = self.attenuation.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(" ");
        elem.push_attribute(("attenuation", attenuation.as_str()));

        if self.refractive_index != [1.0, 1.0, 1.0] {
            let index = self.refractive_index.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(" ");
            elem.push_attribute(("refractiveindex", index.as_str()));
        }

        if self.roughness != 0.0 {
            elem.push_attribute(("roughness", self.roughness.to_string().as_str()));
        }

        writer.write_event(Event::Empty(elem))?;
        Ok(())
    }
}

impl MaterialResources {
    pub fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        for texture in &self.texture_2ds {
            texture.write(writer)?;
        }

        for color_group in &self.color_groups {
            color_group.write(writer)?;
        }

        for texture_group in &self.texture_2d_groups {
            texture_group.write(writer)?;
        }

        for composite in &self.composite_materials {
            composite.write(writer)?;
        }

        for multi_props in &self.multi_properties {
            multi_props.write(writer)?;
        }

        for props in &self.pb_specular_display_properties {
            props.write(writer)?;
        }

        for props in &self.pb_metallic_display_properties {
            props.write(writer)?;
        }

        for props in &self.pb_specular_texture_display_properties {
            props.write(writer)?;
        }

        for props in &self.pb_metallic_texture_display_properties {
            props.write(writer)?;
        }

        for props in &self.translucent_display_properties {
            props.write(writer)?;
        }

        Ok(())
    }
}