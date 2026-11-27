// SPDX-License-Identifier: GPL-3.0-or-later
//! The two independent searchBedLayer walks. Kept dependency-free for native reuse.
use std::collections::{BTreeSet, VecDeque};
pub fn layers(head:[i32;3],facing:[i32;3],max_layers:i32)->Vec<([i32;3],i32)>{
    let opposite=facing.map(|v|-v);
    let left=if facing[0]!=0{[0,0,1]}else{[-1,0,0]};let right=left.map(|v|-v);
    let mut result=Vec::new();
    for(start,outward)in[(head,facing),(std::array::from_fn(|i|head[i].wrapping_add(opposite[i])),opposite)]{
        let mut visited=BTreeSet::from([start]);let mut queue=VecDeque::from([(start,0)]);
        while let Some((pos,layer))=queue.pop_front(){
            if layer>=max_layers{continue;}
            for direction in[outward,[0,1,0],left,right]{
                let next=std::array::from_fn(|i|pos[i].wrapping_add(direction[i]));
                if visited.insert(next){result.push((next,layer+1));queue.push_back((next,layer+1));}
            }
        }
    }
    result
}
