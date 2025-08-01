const Compilation = require("@rspack/core").Compilation;
// const Source = require("webpack-sources").Source;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	plugins: [
		compiler => {
			const files = {};
			compiler.hooks.assetEmitted.tap(
				"Test",
				(file, { content, source, outputPath, compilation, targetPath }) => {
					expect(Buffer.isBuffer(content)).toBe(true);
					// CHANGE: source is instace of RawSource
					// expect(source).toBeInstanceOf(Source);
					expect(typeof outputPath).toBe("string");
					expect(typeof targetPath).toBe("string");
					expect(compilation).toBeInstanceOf(Compilation);

					files[file] = true;
				}
			);
			compiler.hooks.afterEmit.tap("Test", () => {
				expect(files).toMatchInlineSnapshot(`
			Object {
			  "694.bundle0.js": true,
			  "bundle0.js": true,
			}
		`);
			});
		}
	]
};
