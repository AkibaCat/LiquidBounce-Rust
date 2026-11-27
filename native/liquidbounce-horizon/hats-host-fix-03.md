# Hats mutable Host correction, revision 03

Official Check 133-02 found that the generated native `Host::chat` takes
`&mut self`. Image-load errors now drain once in `client_hats_tick(&mut self)`
at the existing ClientTick.Pre hook. The read-only frame snapshot never calls
a mutable host operation. The test declaration provider now matches the real
native signature for future use.

No smoke was repeated. All previous freezes and smoke evidence are retained.
Root owns the next official aggregate Check.
