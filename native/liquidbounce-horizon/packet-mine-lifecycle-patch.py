"""Idempotent PacketMine disable cleanup seam; root-owned integration helper."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
def apply(edit,once):
    def behaviors(source):
        anchor='            ModuleCallback::OnDisabled(id) if self.query_ids.get("LiquidFiller")==Some(&id)=>{self.liquid_filler.placer.disable(current_time_millis());},'
        addition='            ModuleCallback::OnDisabled(id) if self.query_ids.get("PacketMine")==Some(&id)=>{self.packet_mine.reset();},\n'
        return once(source,anchor,addition+anchor)
    edit('behaviors.rs',behaviors)
