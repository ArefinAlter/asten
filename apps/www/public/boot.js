(function () {
  var root = document.documentElement;
  root.classList.add("js");
  try {
    var theme = localStorage.getItem("asten-theme");
    if (theme === "day" || theme === "night") root.setAttribute("data-theme", theme);
  } catch (error) {}
})();
