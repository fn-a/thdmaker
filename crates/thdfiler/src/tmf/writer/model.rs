use std::io::Write;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Writer;
use super::error::Result;
use super::define::package::*;
use super::define::model::*;

impl Model {
    pub fn write<W: Write>(&self, writer: W) -> Result<()> {
        let mut xml_writer = Writer::new_with_indent(writer, b' ', 2);
    
        // XML declaration
        xml_writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;
    
        // Model element
        let mut model_elem = BytesStart::new("model");
        model_elem.push_attribute(("xmlns", Namespace::CORE_NS));
        model_elem.push_attribute(("unit", self.unit.to_string().as_str()));
        
        if let Some(ref lang) = self.language {
            model_elem.push_attribute(("xml:lang", lang.as_str()));
        }
        
        let mut required_extensions = self.required_extensions.clone();

        // Slice Extension
        // A 3MF package containing "lowres" objects MUST list the slice extension in the requiredextensions property of the model section
        // If the mesh resolution is "lowres", the 3MF document MUST list the slice extension in the requiredextension attribute of the model section
        // <model ... xmlns:s="http://schemas.microsoft.com/3dmanufacturing/slice/2015/07" requiredextensions="s">
        if self.resources.slices.is_some() {
            model_elem.push_attribute(("xmlns:s", Namespace::SLICE_NS));
            required_extensions.push("s".to_string());
        }

        if !required_extensions.is_empty() {
            model_elem.push_attribute(("requiredextensions", self.required_extensions.join(" ").as_str()));
        }
        
        if !self.recommended_extensions.is_empty() {
            model_elem.push_attribute(("recommendedextensions", self.recommended_extensions.join(" ").as_str()));
        }
    
        xml_writer.write_event(Event::Start(model_elem))?;
    
        // Metadata
        for meta in &self.metadatas {
            meta.write(&mut xml_writer)?;
        }
    
        // Resources
        self.resources.write(&mut xml_writer)?;
    
        // Build
        self.build.write(&mut xml_writer)?;
    
        // Close model
        xml_writer.write_event(Event::End(BytesEnd::new("model")))?;
    
        Ok(())
    }

}

impl Metadata {
    fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut elem = BytesStart::new("metadata");
        elem.push_attribute(("name", self.name.as_str()));
        
        if self.preserve {
            elem.push_attribute(("preserve", "1"));
        }
        
        if let Some(ref dtype) = self.r#type {
            elem.push_attribute(("type", dtype.as_str()));
        }
    
        writer.write_event(Event::Start(elem))?;
        writer.write_event(Event::Text(BytesText::new(&self.value)))?;
        writer.write_event(Event::End(BytesEnd::new("metadata")))?;
    
        Ok(())
    }
}

impl Resources {
    fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        writer.write_event(Event::Start(BytesStart::new("resources")))?;
    
        // Base materials
        for materials in &self.base_materials {
            materials.write(writer)?;
        }
        
        // Objects
        for object in &self.objects {
            object.write(writer)?;
        }

        // Displacement extension resources
        if let Some(ref displacements) = self.displacements {
            displacements.write(writer)?;
        }

        // Material extension resources
        if let Some(ref materials) = self.materials {
            materials.write(writer)?;
        }

        // Slice extension resources
        if let Some(ref slices) = self.slices {
            slices.write(writer)?;
        }

        // Volumetric extension resources
        if let Some(ref volumetric) = self.volumetric {
            volumetric.write(writer)?;
        }

        // Implicit extension resources
        if let Some(ref implicit) = self.implicit {
            implicit.write(writer)?;
        }
    
        writer.write_event(Event::End(BytesEnd::new("resources")))?;

        Ok(())
    }
}

impl BaseMaterials {
    fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut elem = BytesStart::new("basematerials");
        elem.push_attribute(("id", self.id.to_string().as_str()));
        // Material extension display properties ID
        if let Some(props_id) = self.display_properties_id {
            elem.push_attribute(("displaypropertiesid", props_id.to_string().as_str()));
        }
        writer.write_event(Event::Start(elem))?;
        for material in &self.materials {
            let mut base_elem = BytesStart::new("base");
            base_elem.push_attribute(("name", material.name.as_str()));
            base_elem.push_attribute(("displaycolor", material.display_color.to_string().as_str()));
            writer.write_event(Event::Empty(base_elem))?;
        }
    
        writer.write_event(Event::End(BytesEnd::new("basematerials")))?;

        Ok(())
    }
}

impl Object {
    fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut elem = BytesStart::new("object");
        elem.push_attribute(("id", self.id.to_string().as_str()));
        
        if self.r#type != ObjectType::Model {
            elem.push_attribute(("type", self.r#type.to_string().as_str()));
        }
        
        if let Some(ref name) = self.name {
            elem.push_attribute(("name", name.as_str()));
        }
        
        if let Some(ref partnumber) = self.part_number {
            elem.push_attribute(("partnumber", partnumber.as_str()));
        }
        
        if let Some(ref thumbnail) = self.thumbnail {
            elem.push_attribute(("thumbnail", thumbnail.as_str()));
        }
        
        if let Some(pid) = self.pid {
            elem.push_attribute(("pid", pid.to_string().as_str()));
        }
        
        if let Some(pindex) = self.pindex {
            elem.push_attribute(("pindex", pindex.to_string().as_str()));
        }

        if let Some(ref production) = self.production {
            production.write(&mut elem)?;
        }

        if let Some(ref slice) = self.slice {
            slice.write(&mut elem)?;
        }
    
        writer.write_event(Event::Start(elem))?;
    
        // Metadata group
        if !self.metadata_group.metadatas.is_empty() {
            writer.write_event(Event::Start(BytesStart::new("metadatagroup")))?;
            for meta in &self.metadata_group.metadatas {
                meta.write(writer)?;
            }
            writer.write_event(Event::End(BytesEnd::new("metadatagroup")))?;
        }
    
        // Content
        match &self.content {
            ObjectContent::Mesh(mesh) => mesh.write(writer)?,
            ObjectContent::Components(components) => components.write(writer)?,
            ObjectContent::BooleanShape(boolean_shape) => boolean_shape.write(writer)?,
            ObjectContent::DisplacementMesh(displacement_mesh) =>  displacement_mesh.write(writer)?,
            ObjectContent::Alternatives(alternatives) => alternatives.write(writer)?,
            ObjectContent::LevelSet(levelset) => levelset.write(writer)?,
        }
    
        writer.write_event(Event::End(BytesEnd::new("object")))?;

        Ok(())
    }
}

impl Components {
    fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        writer.write_event(Event::Start(BytesStart::new("components")))?;
    
        for component in &self.components {
            let mut elem = BytesStart::new("component");
            elem.push_attribute(("objectid", component.object_id.to_string().as_str()));
            
            if let Some(ref transform) = component.transform {
                elem.push_attribute(("transform", transform.to_string().as_str()));
            }

            if let Some(ref production) = component.production {
                production.write(&mut elem)?;
            }
            
            writer.write_event(Event::Empty(elem))?;
        }
    
        writer.write_event(Event::End(BytesEnd::new("components")))?;

        Ok(())
    }
}

impl Build {
    fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut build = BytesStart::new("build");
        
        if let Some(ref production) = self.production {
            production.write(&mut build)?;
        }

        writer.write_event(Event::Start(build))?;
    
        for item in &self.items {
            let mut elem = BytesStart::new("item");
            elem.push_attribute(("objectid", item.object_id.to_string().as_str()));
            
            if let Some(ref transform) = item.transform {
                elem.push_attribute(("transform", transform.to_string().as_str()));
            }
            
            if let Some(ref partnumber) = item.part_number {
                elem.push_attribute(("partnumber", partnumber.as_str()));
            }

            if let Some(ref production) = item.production {
                production.write(&mut elem)?;
            }
    
            if item.metadata_group.metadatas.is_empty() {
                writer.write_event(Event::Empty(elem))?;
            } else {
                writer.write_event(Event::Start(elem))?;
                writer.write_event(Event::Start(BytesStart::new("metadatagroup")))?;
                for meta in &item.metadata_group.metadatas {
                    meta.write(writer)?;
                }
                writer.write_event(Event::End(BytesEnd::new("metadatagroup")))?;
                writer.write_event(Event::End(BytesEnd::new("item")))?;
            }
        }
    
        writer.write_event(Event::End(BytesEnd::new("build")))?;

        Ok(())
    }
}