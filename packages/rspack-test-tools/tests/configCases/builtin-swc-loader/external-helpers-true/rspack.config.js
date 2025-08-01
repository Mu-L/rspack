/** @type {import("@rspack/core").Configuration} */
module.exports = {
	resolve: {
		extensions: ["...", ".ts"]
	},
	mode: "development",
	module: {
		rules: [
			{
				test: /\.ts$/,
				use: [
					{
						loader: "builtin:swc-loader",
						options: {
							jsc: {
								externalHelpers: true,
								target: "es5"
							}
						}
					}
				],
				type: "javascript/auto"
			}
		]
	}
};
