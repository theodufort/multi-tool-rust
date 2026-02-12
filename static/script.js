document.addEventListener("DOMContentLoaded", () => {
  const STORAGE_KEY = "sidebar-collapsed";

  const toggle = document.getElementById("sidebar-toggle");
  const sidebar = document.getElementById("sidebar");

  if (toggle && sidebar) {
    if (localStorage.getItem(STORAGE_KEY) === "true") {
      document.body.classList.add("sidebar-collapsed");
    }

    toggle.addEventListener("click", () => {
      document.body.classList.toggle("sidebar-collapsed");
      localStorage.setItem(STORAGE_KEY, document.body.classList.contains("sidebar-collapsed"));
    });
  }

  document.querySelectorAll(".sidebar .nav-link").forEach((link) => {
    link.addEventListener("click", (e) => {
      if (link.getAttribute("href") === "#") {
        e.preventDefault();
      }
      document.querySelectorAll(".sidebar .nav-link.active").forEach((active) => {
        active.classList.remove("active");
      });
      link.classList.add("active");
    });
  });
});
