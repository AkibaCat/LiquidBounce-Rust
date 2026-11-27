# EasyPearl revision 02

Official grouped check 17004 found two invalid `PlayerMotion::eye()` calls. Both now read the actual `PlayerMotion.dimensions.eye_height`: the OUTLINE look ray keeps interpolated previous/current feet plus current pose eye height, and the distance label remains at hit plus current eye height.

This only repairs the two owned native expressions. Historical static and official reports remain intact. No Rust, Cargo, renderer matrix, or game execution was repeated for this correction. The next check is root's grouped official retry.
