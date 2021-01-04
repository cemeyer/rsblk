#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use anyhow::Result;
use freebsd_geom::{self as geom, GeomClass};
use ptree::{self, print_config::PrintConfig, TreeBuilder};
use std::{
    self,
    cmp::Ordering,
    fmt::{self, Display, Formatter},
    io::Write,
    str::FromStr,
    string::ToString,
};
use strum_macros::Display as StrumDisplay;
use tabwriter::TabWriter;

#[derive(StrumDisplay, Debug)]
enum Col {
    #[strum(serialize = "NAME")]
    Name,
    #[strum(serialize = "CLASS")]
    Class,
    #[strum(serialize = "SIZE")]
    Size,
    #[strum(serialize = "FSTYPE")]
    FsType,
    #[strum(serialize = "LABEL")]
    Label,
    #[strum(serialize = "MOUNTPOINT")]
    Mountpoint,
    #[strum(serialize = "UUID")]
    Uuid,
}

fn emit_header(w: &mut TabWriter<Vec<u8>>, cols: &Vec<Col>) -> Result<()> {
    write!(
        w,
        "{}\n",
        cols.iter()
            .map(|c| format!("{}", c))
            .collect::<Vec<String>>()
            .join("\t")
    )?;
    Ok(())
}

fn format_root(name: &str, ncols: usize) -> String {
    let pad = if ncols > 1 {
        "\t".repeat(ncols - 1)
    } else {
        "".to_owned()
    };
    return format!("{}{}", name, pad);
}

fn format_datum(forest: &geom::Graph, edge: &geom::Edge, node: &geom::Geom, col: &Col) -> String {
    match col {
        Col::Name => edge.name.to_owned(),
        Col::Class => format!("{:?}", &node.class),
        Col::Size => format!("{}", edge.mediasize),
        Col::FsType => match &edge.metadata {
            Some(edgemd) => match &**edgemd {
                geom::EdgeMetadata::PART { type_: ptype, .. } => ptype.to_owned(),
                _ => "".to_owned(),
            },
            _ => "".to_owned(),
        },
        Col::Label => match &edge.metadata {
            Some(edgemd) => match &**edgemd {
                geom::EdgeMetadata::PART { label: plabel, .. } => {
                    plabel.as_deref().unwrap_or("").to_owned()
                }
                _ => "".to_owned(),
            },
            _ => "".to_owned(),
        },
        // Col::Mountpoint will require looking at the mountpoint list and maybe recursive search
        // for SWAP geoms under any matching edge (e.g., label or dev child).
        Col::Uuid => match &edge.metadata {
            Some(edgemd) => match &**edgemd {
                geom::EdgeMetadata::PART { rawuuid: uuid, .. } => {
                    uuid.as_deref().unwrap_or("").to_owned()
                }
                _ => "".to_owned(),
            },
            _ => "".to_owned(),
        },

        // Not yet implemented:
        _ => "".to_owned(),
    }
}

fn walk_geom(
    forest: &geom::Graph,
    nodeid: &geom::NodeId,
    node: &geom::Geom,
    cols: &Vec<Col>,
    tb: &mut TreeBuilder,
) {
    if node.class == GeomClass::LABEL {
        return;
    }

    // Acquire a vec of children, sorted by name.  (Lexographic for now, although it would be nice
    // to order numbers numerically rather than lexographically eventually.)
    let mut children: Vec<_> = forest
        .child_geoms_iter(nodeid)
        .map(|(_, edge, child)| (edge, child))
        .collect();
    children.sort_by_key(|(e1, _)| &e1.name);

    for (edge, child) in children.iter() {
        let mut addnode = false;
        if node.class == GeomClass::PART && child.class == GeomClass::DEV {
            addnode = true;
        }

        if addnode {
            tb.begin_child(
                cols.iter()
                    .map(|c| format_datum(forest, edge, child, c))
                    .collect::<Vec<String>>()
                    .join("\t"),
            );
        }

        walk_geom(forest, &edge.consumer_geom, child, cols, tb);

        if addnode {
            tb.end_child();
        }
    }
}

// TODO:
// 1. Options parsing: [options] [device...]
fn run() -> Result<()> {
    let cols = vec![Col::Name, Col::FsType, Col::Label, Col::Uuid];

    let mut tw = TabWriter::new(vec![]);
    emit_header(&mut tw, &cols)?;

    let config = PrintConfig {
        indent: 2,
        padding: 0,
        ..PrintConfig::from_env()
    };

    let forest = geom::get_graph()?;
    // Sort the roots by name.  (Lexographic for now, although human-sort eventually.)
    let mut roots: Vec<_> = forest.roots_iter().collect();
    roots.sort_by_key(|(_, g)| &g.name);

    for (nodeid, node) in roots.iter() {
        let mut tb = TreeBuilder::new(format_root(&node.name, cols.len()));
        walk_geom(&forest, nodeid, node, &cols, &mut tb);
        let tree = tb.build();
        ptree::write_tree_with(&tree, &mut tw, &config)?;
    }

    tw.flush()?;
    print!("{}", String::from_utf8(tw.into_inner()?)?);
    return Ok(());
}

fn main() {
    run().unwrap();
}
