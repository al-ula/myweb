function theme() {
    const doc = document.documentElement as HTMLHtmlElement;
    const theme = doc.getAttribute("data-theme") as string;
    const controller = document.getElementsByClassName("theme-controller")[0] as HTMLInputElement;
    let localTheme = localStorage.getItem("theme") as string;
    let isToggled = controller.checked;
    if (localTheme === theme) {
        controller.checked= false;
    } else if (localTheme === controller.value) {
        controller.checked= true;
    } else {
        controller.checked= false;
    }
    controller.addEventListener("click", () => {
        isToggled = controller.checked;
        if (isToggled === true) {
            localStorage.setItem("theme", controller.value);
        } else {
            localStorage.setItem("theme", theme);
        }
    })
}
theme()