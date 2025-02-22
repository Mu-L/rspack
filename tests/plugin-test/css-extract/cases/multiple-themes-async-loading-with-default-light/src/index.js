/* eslint-env browser */
import "./style.scss";

let theme = "light";
const themes = {};

themes[theme] = document.querySelector("#theme");

async function loadTheme(newTheme) {
	// eslint-disable-next-line no-console
	console.log(`CHANGE THEME - ${newTheme}`);

	const themeElement = document.querySelector("#theme");

	if (themeElement) {
		themeElement.remove();
	}

	if (themes[newTheme]) {
		// eslint-disable-next-line no-console
		console.log(`THEME ALREADY LOADED - ${newTheme}`);

		document.head.appendChild(themes[newTheme]);

		return;
	}

	if (newTheme === "dark") {
		// eslint-disable-next-line no-console
		console.log(`LOADING THEME - ${newTheme}`);

		// eslint-disable-next-line import/no-unresolved
		import(/* webpackChunkName: "dark" */ "./style.scss?dark").then(() => {
			themes[newTheme] = document.querySelector("#theme");

			// eslint-disable-next-line no-console
			console.log(`LOADED - ${newTheme}`);
		});
	}
}

document.onclick = () => {
	if (theme === "light") {
		theme = "dark";
	} else {
		theme = "light";
	}

	loadTheme(theme);
};
