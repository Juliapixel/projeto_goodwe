window.addEventListener("load", () => {
    let query = document.getElementsByClassName("togglador")
    for (let i = 0; i < query.length; i++) {
        const element = query.item(i);
        element.addEventListener("change", toggleModo)
    }
})

function toggleModo(event) {
    console.log(event)
    // elemento que clicamos
    event.target;
    if (event.target.checked) {
        fetch(`/api/setstatus?id=338c1c8a-c3a2-4715-be92-8911248bbb8c&state=on`, { method: "POST" })
    } else {
        fetch(`/api/setstatus?id=338c1c8a-c3a2-4715-be92-8911248bbb8c&state=on`, { method: "POST" })
    }
}