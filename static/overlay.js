function i(){const t=document.getElementById("side-btn"),e=document.getElementById("overlay-wrapper"),s=document.getElementById("overlay-aside");t&&e&&s&&(t.addEventListener("click",()=>{e.classList.toggle("hidden"),e.classList.toggle("grid")}),e.addEventListener("click",n=>{const d=n.target;!s.contains(d)&&!e.classList.contains("hidden")&&(e.classList.add("hidden"),e.classList.remove("grid"))}))}i();