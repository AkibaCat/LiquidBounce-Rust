# ProphuntESP / VoidESP revision 02

The transactional integrator now adds ProphuntESP's source aliases BlockUpdateDetector and FallingBlockESP to the functioning ModuleKind::from_name lookup. Pinned catalog metadata already contained them, but a functioning module requires the explicit name-resolution mapping too. Both refer to the same ProphuntEsp ModuleKind. Behavior, config trees, runtime and native code are unchanged; no repeated smoke was run. Use placement-esp-freeze-02.json and preserve the original freeze and smoke report.
