/** @type {import('tailwindcss').Config} */
import daisyui from "daisyui";
import catppuccin from "@catppuccin/daisyui";
import typography from "@tailwindcss/typography";

export default {
  content: ["./templates/**/*.{html,hbs}", "./src/**/*.rs"],
  theme: {
    extend: {
      fontFamily: {
        hasklig: ["Hasklig"],
      },
    },
  },
  plugins: [typography, daisyui],
  daisyui: {
    themes: ["light", "dark", catppuccin("latte"), catppuccin("mocha")],
  },
}

