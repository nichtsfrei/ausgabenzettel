const labels = [
  {
    title: "Groceries",
    description: "Food & drinks for home",
    color: "#5f90b0",
    index: 0,
  },
  {
    title: "Dining Out",
    description: "Restaurants, cafés, bars, takeout, delivery",
    color: "#6068af",
    index: 1,
  },
  {
    title: "Housing",
    description: "Rent, utilities, maintenance, repairs",
    color: "#60afa6",
    index: 2,
  },
  {
    title: "Transportation",
    description: "Public transit, fuel, car costs, bike, parking, rideshare",
    color: "#7f62ad",
    index: 3,
  },
  {
    title: "Necessities",
    description:
      "Essential non-food items like cleaning supplies, toiletries, basic clothing",
    color: "#62ad7f",
    index: 4,
  },
  {
    title: "Entertainment",
    description:
      "Electronics, gadgets, entertainment, hobbies, subscriptions, leisure activities",
    color: "#a463ac",
    index: 5,
  },
];

function readableTextColor(hex) {
  hex = hex.replace("#", "");

  let r = parseInt(hex.substring(0, 2), 16) / 255;
  let g = parseInt(hex.substring(2, 4), 16) / 255;
  let b = parseInt(hex.substring(4, 6), 16) / 255;

  r = r <= 0.03928 ? r / 12.92 : Math.pow((r + 0.055) / 1.055, 2.4);
  g = g <= 0.03928 ? g / 12.92 : Math.pow((g + 0.055) / 1.055, 2.4);
  b = b <= 0.03928 ? b / 12.92 : Math.pow((b + 0.055) / 1.055, 2.4);

  const L = 0.2126 * r + 0.7152 * g + 0.0722 * b;

  return L > 0.179 ? "#000000" : "#ffffff";
}

window.onload = function () {
  updateEntries();
};

function createAgenda(labels) {
  const container = document.getElementById("label_agenda");
  container.innerHTML = "";
  labels.sort((a, b) => a.value < b.value);

  labels.forEach((label) => {
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

    // Apply your dynamic-color class and CSS variables
    details.classList.add("dynamic-color");
    details.style.setProperty("--bg", label.color);
    details.style.setProperty("--fg", readableTextColor(label.color));

    container.appendChild(details);
  });
}

function createEntryTemplate(entry, index) {
  const details = document.createElement("details");

  const titleSpan = document.createElement("span");
  titleSpan.textContent = entry.label;
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
    removeEntry(index);
  });

  details.appendChild(summary);
  details.appendChild(removeLink);
  details.setAttribute("data-timestamp", entry.timestamp);

  return details;
}

function removeEntry(index) {
  const storedData = JSON.parse(localStorage.getItem("dailyEntries")) || [];

  storedData.splice(index, 1);
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
  labels.forEach((label) => {
    const option = document.createElement("option");
    option.textContent = label.title;
    option.value = label.index;
    details.appendChild(option);
  });
}

function updateEntries() {
  prepareDropDown(labels);
  const details = document.getElementById("details");
  const storedData = JSON.parse(localStorage.getItem("dailyEntries")) || [];

  details.innerHTML = "";

  // const newEntry = {
  //   value: Number(value).toFixed(2),
  //   currency: currency,
  //   label: label, // index
  //   timestamp: Math.floor(Date.now() / 1000),
  // };

  const prepared_labels = labels.map((label) => {
    label.value = 0;
    return label;
  });
  const summaries = storedData.reduce((p, c) => {
    console.log(p, c);
    let idx = p.findIndex((item) => item.index === Number(c.label));
    let e = p[idx];
    e.value += Number(c.value);
    p[idx] = e;
    return p;
  }, prepared_labels);

  createAgenda(summaries);
  if (storedData.length === 0) {
    details.innerHTML = "<p>No entries yet</p>";
  } else {
    const titled = storedData.map((e) => {
      let label = labels.find((item) => item.index === Number(e.label));
      e.label = label.title;
      return e;
    });
    const pie_size = convertEmToPx(summaries.length);
    const colors = ["#80a8cc", "#da3b3e", "#ffa921", "red"];
    const don = donut({
      data: summaries,
      size: pie_size,
      weight: pie_size / 2,
    });
    document.getElementById("daily_donut").innerHTML = don.innerHTML;

    const offset = storedData.length - 1;
    titled.reverse().forEach((e, index) => {
      const entry = createEntryTemplate(e, offset - index);
      details.appendChild(entry);
    });
  }
}

document.getElementById("daily_form").addEventListener("submit", function () {
  event.preventDefault();
  const value = document.getElementById("daily_input").value;
  const currency = "€";
  const label = document.getElementById("daily_label_select").value;

  if (value && label) {
    const dailyEntries = JSON.parse(localStorage.getItem("dailyEntries")) || [];
    const newEntry = {
      value: Number(value).toFixed(2),
      currency: currency,
      label: label, // index
      timestamp: Math.floor(Date.now() / 1000),
    };
    dailyEntries.push(newEntry);
    localStorage.setItem("dailyEntries", JSON.stringify(dailyEntries));
    document.getElementById("daily_input").value = "";
    document.getElementById("daily_label_select").value = label;
    updateEntries();
  }
  document.getElementById("daily_input").focus();
});
