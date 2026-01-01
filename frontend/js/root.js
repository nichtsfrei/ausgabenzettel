class Label {
  constructor(title, description, index) {
    this.index = Number(index);
    this.title = title;
    this.description = description;
  }
  get toClass() {
    return Label.toClass(this.index);
  }
  static toClass(index) {
    return `cat${Number(index) + 1}`;
  }

  static fromClass(cl) {
    if (!cl.startsWith("cat")) return null;
    let cidx = cl.substring(3); // cat
    return Number(cidx) - 1;
  }
}

const labels = [
  new Label("Groceries", "Food & drinks for home", 0),
  new Label("Dining Out", "Restaurants, cafés, bars, takeout, delivery", 1),
  new Label("Housing", "Rent, utilities, maintenance, repairs", 2),
  new Label(
    "Transportation",
    "Public transit, fuel, car costs, bike, parking, rideshare",
    3,
  ),
  new Label(
    "Necessities",
    "Essential non-food items like cleaning supplies, toiletries, basic clothing",
    4,
  ),
  new Label(
    "Entertainment",
    "Electronics, gadgets, entertainment, hobbies, subscriptions, leisure activities",
    5,
  ),
];

class Entry {
  constructor(value, currency, label, timestamp) {
    this.value = Number(value).toFixed(2);
    this.currency = currency;
    this.label = Number(label);
    this.timestamp = Number(timestamp);
  }
}

class Filter {
  constructor() {
    // <select id="menu_overview_select">
    let mos = document.getElementById("menu_overview_select").value || "daily";
    let selectedDate =
      document.getElementById("daily_date").valueAsDate || new Date();
    this.filter = mos;
    this.selectedDate = selectedDate;
  }

  show(entry) {
    if (this.filter == "all") {
      return true;
    }
    let ed = new Date(entry.timestamp);
    if (this.filter == "daily") {
      return (
        this.selectedDate.getDate() == ed.getDate() &&
        this.selectedDate.getMonth() == ed.getMonth() &&
        this.selectedDate.getYear() == ed.getYear()
      );
    }
    if (this.filter == "weekly") {
      return (
        this.selectedDate.getYear() == ed.getYear() &&
        this.selectedDate.getWeek() == ed.getWeek()
      );
    }
    if (this.filter == "monthly") {
      return (
        this.selectedDate.getMonth() == ed.getMonth() &&
        this.selectedDate.getYear() == ed.getYear()
      );
    }
    if (this.filter == "yearly") {
      return this.selectedDate.getYear() == ed.getYear();
    }
    return false;
  }
}

window.onload = function () {
  addDocumentEventListener();
  storeCurrentEtag();
  prepareDropDown(labels);
  updateEntries();
};

function storeEtag(response) {
  let etag = response.headers.get("etag");
  localStorage.setItem("etag", etag);
}

function storeCurrentEtag() {
  if (window.location.href.startsWith("file://")) return;
  fetch("/", {
    method: "HEAD",
  })
    .then((response) => {
      if (response.status === 200) {
        storeEtag(response);
      } else {
        console.log("unexpected status", response);
      }
    })
    .catch((error) => {
      console.log("Error:", error);
    });
}

function createAgenda(total, labels) {
  const container = document.getElementById("label_agenda");
  container.innerHTML = "";
  labels.sort((a, b) => a.value < b.value);

  let createDetail = (label) => {
    const details = document.createElement("details");
    const summary = document.createElement("summary");
    const currency = "€";

    const titleSpan = document.createElement("span");
    titleSpan.textContent = label.title;
    const currencySpan = document.createElement("span");
    currencySpan.textContent = `${label.value.toFixed(2)}${currency}`;
    summary.appendChild(titleSpan);
    summary.appendChild(currencySpan);

    const paragraph = document.createElement("p");
    paragraph.textContent = label.description;

    details.appendChild(summary);
    details.appendChild(paragraph);

    details.classList.add(label.toClass);
    return details;
  };

  labels.forEach((label) => {
    container.appendChild(createDetail(label));
  });

  container.appendChild(
    createDetail({ index: -1, title: "Total", value: total }),
  );
}

function createEntryTemplate(show, label, entry) {
  const details = document.createElement("details");
  details.classList.add(Label.toClass(entry.label));
  details.id = entry.timestamp;

  const titleSpan = document.createElement("span");
  titleSpan.textContent = label.title;
  const currencySpan = document.createElement("span");
  currencySpan.textContent = `${entry.value}${entry.currency}`;

  const summary = document.createElement("summary");
  summary.appendChild(titleSpan);
  summary.appendChild(currencySpan);

  const removeLink = document.createElement("a");
  removeLink.textContent = "remove";
  removeLink.href = "#";
  removeLink.addEventListener("click", function (e) {
    e.preventDefault();
    removeEntry(details.id);
  });

  details.appendChild(summary);
  details.appendChild(removeLink);

  if (!show) {
    details.classList.add("hidden");
  }

  return details;
}

function removeEntry(timestamp) {
  const storedData = JSON.parse(localStorage.getItem("dailyEntries")) || [];

  let index = storedData.findIndex((element) => {
    return element.timestamp == timestamp;
  });
  if (index > -1) {
    storedData.splice(index, 1);
    document.getElementById(timestamp).remove();
  } else {
    const newEntry = {
      event: "remove",
      timestamp: timestamp,
    };
    storedData.push(newEntry);
  }

  localStorage.setItem("dailyEntries", JSON.stringify(storedData));

  updateEntries();
}

function convertEmToPx(emValue, context) {
  const fontSize = parseFloat(
    getComputedStyle(context || document.documentElement).fontSize,
  );
  return emValue * fontSize;
}

function prepareDropDown(labels) {
  const details = document.getElementById("daily_label_select");
  details.innerHTML = "";
  labels.forEach((label) => {
    const option = document.createElement("option");
    option.textContent = label.title;
    option.value = label.index;
    details.appendChild(option);
  });
}

function getHTMLEntries(cached) {
  function splitCurrencyLabel(label) {
    let i = label.length - 1;
    const value_symbols = "0123456789,.";
    while (i >= 0 && !value_symbols.includes(label[i])) i--;
    return {
      currency: label.slice(i + 1).trim(),
      value: label.slice(0, i + 1),
    };
  }
  const details = document.getElementById("details").children;
  const results = [];
  for (let i = 0; i < details.length; ++i) {
    let timestamp = details[i].id;
    let label = [...details[i].classList]
      .map(Label.fromClass)
      .find((x) => x != null);
    let vc = splitCurrencyLabel(details[i].children[0].children[1].textContent);
    const entry = new Entry(vc.value, vc.currency, label, timestamp);

    if (cached.findIndex((e) => e.timestamp == entry.timestamp) == -1) {
      results.push(entry);
    }
  }

  return results;
}

function getRootColor(color) {
  return getComputedStyle(document.documentElement)
    .getPropertyValue(color)
    .trim();
}

var agenda_size; // is set once on updateEntries and used for the donut size

function updateEntries() {
  const details = document.getElementById("details");
  const cachedData = JSON.parse(localStorage.getItem("dailyEntries")) || [];
  const storedData = getHTMLEntries(cachedData);
  storedData.push(...cachedData);

  //TODO: don't redraw on cachedData.length === 0 but add remove listener
  details.innerHTML = "";
  const prepared_labels = labels.map((label) => {
    label.value = 0;
    return label;
  });

  const dont_draw = storedData
    .filter((e) => e.event === "remove")
    .map((e) => e.timestamp);
  const drawData = storedData.filter((e) => e.event !== "remove");
  const filtered = new Filter();
  const colors = [
    getRootColor("--cat1"),
    getRootColor("--cat2"),
    getRootColor("--cat3"),
    getRootColor("--cat4"),
    getRootColor("--cat5"),
    getRootColor("--cat6"),
  ];
  var total = 0;
  const summaries = drawData.reverse().reduce((p, c) => {
    if (!dont_draw.includes(c.timestamp)) {
      let idx = p.findIndex((item) => item.index === Number(c.label));
      let show = filtered.show(c);
      let e = p[idx];
      if (show) {
        e.value += Number(c.value);
        total += Number(c.value);
        e.color = colors[idx];
        p[idx] = e;
      }
      const entry = createEntryTemplate(show, e, c);
      details.appendChild(entry);
    }
    return p;
  }, prepared_labels);

  createAgenda(total, summaries);
  if (!agenda_size) {
    let padding = summaries.length * 0.25 * 2;
    let margin = summaries.length * 0.5;
    agenda_size = convertEmToPx(summaries.length + padding + margin);
  }
  const don = donut({
    data: summaries,
    size: agenda_size,
  });
  document.getElementById("daily_donut").innerHTML = don.innerHTML;
}
function addDocumentEventListener() {
  if (document.getElementById("daily_date").valueAsDate === null)
    document.getElementById("daily_date").valueAsDate = new Date();
  document.getElementById("daily_date").addEventListener("change", function () {
    updateEntries();
  });
  document
    .getElementById("menu_overview_select")
    .addEventListener("change", function () {
      updateEntries();
    });

  document
    .getElementById("daily_label_select")
    .addEventListener("change", function () {
      document.getElementById("daily_input").focus();
    });

  document.getElementById("daily_form").addEventListener("submit", function () {
    event.preventDefault();
    const value = document.getElementById("daily_input").value;
    const currency = "€";
    const label = document.getElementById("daily_label_select").value;

    if (value && label) {
      let selectedDate =
        document.getElementById("daily_date").valueAsDate || new Date();
      let currentTime = new Date();
      selectedDate.setHours(currentTime.getHours());
      selectedDate.setMinutes(currentTime.getMinutes());
      selectedDate.setSeconds(currentTime.getSeconds());
      const dailyEntries =
        JSON.parse(localStorage.getItem("dailyEntries")) || [];
      const newEntry = new Entry(
        value,
        currency,
        label,
        selectedDate.getTime(),
      );
      dailyEntries.push(newEntry);
      localStorage.setItem("dailyEntries", JSON.stringify(dailyEntries));
      document.getElementById("daily_input").value = "";
      document.getElementById("daily_label_select").value = label;
      updateEntries();
    }
    document.getElementById("daily_input").focus();
  });

  document
    .getElementById("menu_export")
    .addEventListener("submit", function () {
      event.preventDefault();
      const target = document.getElementById("menu_export_select").value;

      const htmlContent =
        "<!doctype html>" + document.documentElement.outerHTML;
      if (target === "server") {
        if (window.location.href.startsWith("file://")) return;
        fetch("/", {
          method: "PUT",
          headers: {
            "Content-Type": "text/html",
            "if-match": localStorage.getItem("etag"),
          },
          body: htmlContent,
        })
          .then((response) => {
            if (response.status === 200) {
              localStorage.clear();
            }
            storeEtag(response);
            document.innerHTML = response.text();
          })
          .catch((error) => {
            console.log("Error:", error);
          });
      } else if (target === "file") {
        const blob = new Blob([htmlContent], { type: "text/html" });
        const link = document.createElement("a");
        link.href = URL.createObjectURL(blob);
        link.download = "ausgabenzettel.html";
        link.click();
        localStorage.clear();
      }
    });
}
