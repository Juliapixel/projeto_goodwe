// #sidebar

const openBtn = document.getElementById('open_btn');
const sidebar = document.getElementById('sidebar');

  openBtn.addEventListener("click", () => {
  sidebar.classList.toggle("open-sidebar");
});

