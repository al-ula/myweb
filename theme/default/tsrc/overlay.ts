function overlay() {
  const sideBtn = document.getElementById("side-btn") as HTMLButtonElement;
  const overlayWrapper = document.getElementById(
    "overlay-wrapper"
  ) as HTMLDivElement;
  const overlayAside = document.getElementById(
    "overlay-aside"
  ) as HTMLDivElement;

  if (sideBtn && overlayWrapper && overlayAside) {
    sideBtn.addEventListener("click", () => {
      overlayWrapper.classList.toggle("hidden");
      overlayWrapper.classList.toggle("grid");
    });

    overlayWrapper.addEventListener("click", (e) => {
      const target = e.target as Node;
      if (
        !overlayAside.contains(target) &&
        !overlayWrapper.classList.contains("hidden")
      ) {
        overlayWrapper.classList.add("hidden");
        overlayWrapper.classList.remove("grid");
      }
    });
  }
}
overlay();
