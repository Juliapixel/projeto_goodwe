// #sidebar
document.getElementById('open_btn').addEventListener('click', function(){
    document.getElementById('sidebar').classList.toggle('open-sidebar');
});

const open_btn = document.getElementById("openSearch");
const search_block = document.getElementById("search_block");

open_btn.addEventListener("click", () => {
  search_block.classList.toggle("active");
});

