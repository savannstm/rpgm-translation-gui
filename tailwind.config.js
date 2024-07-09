/** @type {import('tailwindcss').Config} */
module.exports = {
    content: [
        "./src/main.html",
        "./src/hotkeys.html",
        "./src/help.html",
        "./src/about.html",
        "./src/main.ts",
        "./src/options.html",
        "./src/options.ts",
        "./src/loading.html",
        "./src/loading.ts",
    ],
    theme: {
        extend: {},
        fontFamily: {
            material: ["Material Symbols Outlined"],
        },
    },
    plugins: [],
};
