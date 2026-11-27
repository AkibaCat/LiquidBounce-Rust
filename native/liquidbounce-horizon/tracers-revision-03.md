# Tracers revision 03

Aggregate native check 116-01 found two Mob API mismatches in the owned native bridge. Ordinary mob scoreboard identity now reads the actual persistent UUID table by mob id, matching the existing armor-stand/dragon bridge. Baby state now calls Mob::is_baby(). CombineMobs grouping also excludes mannequin from the Mob group, matching the established native CombineMobs adapter. The original freeze and revisions 02 remain preserved. Per root instruction, no smoke or aggregate check was repeated; root owns native restaging and verification.
