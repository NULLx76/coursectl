# Gitlab Helper
A command-line utility for interacting with Gitlab from a course organization perspective

### Grabbing a list of Students from Brightspace
```js
JSON.stringify({ Students: Array.from(document.getElementsByTagName("tr")).map(i => {
  let c = Array.from(i.children).filter(i => i.classList.contains("d_gn")).map(i => i.innerHTML);
    return [c[1], c[2], c[3]]
}).filter(i => i.length != 0).filter(i => typeof i[0] !== "undefined")
.map(i => [i[0].replaceAll(/<\/?label>/g, ""), i[1].replaceAll(/<\/?label>/g, ""), i[2].replaceAll(/<\/?label>/g, "")])
.map(i => {return {OrgDefinedID: i[0], StudentNumber: i[1], email: i[2]}})})
```
