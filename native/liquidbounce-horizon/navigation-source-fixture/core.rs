use crate::navigation::{astar::{AStar,ShortestPath,WeightedEdge},input::{Input,Keystrokes}};
fn path(value:Option<&ShortestPath<usize>>)->String {
    value.map_or_else(||"-".into(),|value|format!("{}:{}",value.nodes.iter().map(usize::to_string).collect::<Vec<_>>().join(","),value.total_cost.to_bits()))
}
#[test]
fn unchanged_agent_astar_and_keystroke_oracle(){
    let directory=std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let commands=std::fs::read_to_string(directory.join("commands.txt")).unwrap();let expected=std::fs::read_to_string(directory.join("results.txt")).unwrap();
    assert_eq!(commands.lines().count(),expected.lines().count());
    let mut keys=Keystrokes::default();let mut count=0;let mut goal=0;let mut heuristics=Vec::new();let mut graph:Vec<Vec<WeightedEdge<usize>>>=Vec::new();let mut search=None;
    for(index,(command,expected))in commands.lines().zip(expected.lines()).enumerate(){
        let p:Vec<_>=command.split(' ').collect();let integer=|i:usize|p[i].parse::<i32>().unwrap();let node=|i:usize|p[i].parse::<usize>().unwrap();let number=|v:&str|f64::from_bits(v.parse::<u64>().unwrap());let mut returned="-".to_owned();
        match p[0]{
            "key"=>returned=keys.press(Input::from_mask(integer(1)),p[2].parse().unwrap(),integer(3),integer(4)).java_hash().to_string(),
            "resetKeys"=>keys.reset(),
            "graph"=>{
                count=node(1);goal=node(3);heuristics=p[5].split(',').map(number).collect();graph=vec![Vec::new();count];
                if p[6]!="-"{for edge in p[6].split(';'){let parts:Vec<_>=edge.split(',').collect();graph[parts[0].parse::<usize>().unwrap()].push(WeightedEdge{node:parts[1].parse().unwrap(),cost:number(parts[2])});}}
                search=Some(AStar::new(node(2),heuristics[node(2)],number(p[4])));
            }
            "advance"=>returned=search.as_mut().unwrap().advance(integer(1),|&n|n==goal,|&n|graph[n].clone(),|&n|heuristics[n]).map_or_else(|_|"invalid".into(),|found|path(found.as_ref())),
            _=>panic!("{command}"),
        }
        if !matches!(p[0],"key"|"resetKeys"){
            let search=search.as_ref().unwrap();
            returned=format!("{returned}|{}|{}|{}|{}|{}",search.iterations,search.is_finished(),path(search.closest_path().as_ref()),(0..count).map(|n|search.cost(&n).to_bits().to_string()).collect::<Vec<_>>().join(","),(0..count).map(|n|search.previous(&n).map_or_else(||"-".into(),usize::to_string)).collect::<Vec<_>>().join(","));
        }
        assert_eq!(returned,expected,"case {index}: {command}");
    }
}
