/** @type {import('../..').TStatsAPICaseConfig} */
module.exports = {
	description: "should have time log when logging verbose",
	options(context) {
		return {
			context: context.getSource(),
			entry: "./fixtures/abc"
		};
	},
	async check(stats) {
		expect(
			stats
				?.toString({ all: false, logging: "verbose" })
				.replace(/\d+ ms/g, "X ms")
		).toMatchInlineSnapshot(`
		LOG from rspack.Compilation
		<t> finish modules: X ms
		<t> optimize dependencies: X ms
		<t> rebuild chunk graph: X ms
		<t> create chunks: X ms
		<t> optimize: X ms
		<t> module ids: X ms
		<t> chunk ids: X ms
		<t> optimize code generation: X ms
		<t> code generation: X ms
		<t> runtime requirements.modules: X ms
		<t> runtime requirements.chunks: X ms
		<t> runtime requirements.entries: X ms
		<t> runtime requirements: X ms
		<t> hashing: hash chunks: X ms
		<t> hashing: hash runtime chunks: X ms
		<t> hashing: process full hash chunks: X ms
		<t> hashing: X ms
		<t> create module assets: X ms
		<t> create chunk assets: X ms
		<t> process assets: X ms
		<t> after process assets: X ms
		<t> after seal: X ms

		LOG from rspack.Compiler
		<t> make hook: X ms
		<t> make: X ms
		<t> finish make hook: X ms
		<t> finish compilation: X ms
		<t> seal compilation: X ms
		<t> emitAssets: X ms

		LOG from rspack.EnsureChunkConditionsPlugin
		<t> ensure chunk conditions: X ms

		LOG from rspack.ModuleConcatenationPlugin
		<t> select relevant modules: X ms
		<t> sort relevant modules: X ms
		<t> find modules to concatenate: X ms
		<t> sort concat configurations: X ms

		LOG from rspack.RealContentHashPlugin
		<t> hash to asset names: X ms

		LOG from rspack.RemoveEmptyChunksPlugin
		<t> remove empty chunks: X ms

		LOG from rspack.SideEffectsFlagPlugin
		<t> prepare connections: X ms
		<t> find optimizable connections: X ms
		<t> do optimize connections: X ms
		<t> update connections: X ms
		    optimized 0 connections

		LOG from rspack.SplitChunksPlugin
		<t> prepare module group map: X ms
		<t> ensure min size fit: X ms
		<t> process module group map: X ms
		<t> ensure max size fit: X ms
	`);
	}
};
