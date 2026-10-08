(function () {
  "use strict";
  document.documentElement.classList.add("js");
  window.I18n.apply("en");

  // Scroll reveal for cards. Without IntersectionObserver everything is simply shown.
  var reveals = document.querySelectorAll(".reveal");
  if ("IntersectionObserver" in window) {
    var io = new IntersectionObserver(function (entries) {
      entries.forEach(function (entry) {
        if (!entry.isIntersecting) return;
        entry.target.classList.add("is-visible");
        io.unobserve(entry.target);
      });
    }, { threshold: 0.15 });
    reveals.forEach(function (el) { io.observe(el); });
  } else {
    reveals.forEach(function (el) { el.classList.add("is-visible"); });
  }

  // Back to top: shown after scrolling; focus moves to the brand link so keyboard users
  // are not left on a button that has just been hidden.
  var toTop = document.querySelector("[data-to-top]");
  if (toTop) {
    var update = function () { toTop.classList.toggle("is-shown", window.scrollY > 600); };
    window.addEventListener("scroll", update, { passive: true });
    update();
    toTop.addEventListener("click", function () {
      window.scrollTo(0, 0);   // honours CSS scroll-behavior (instant under reduced motion)
      var brand = document.querySelector(".site-nav__brand");
      if (brand) brand.focus({ preventScroll: true });
    });
  }
})();