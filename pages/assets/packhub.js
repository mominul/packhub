const body = document.body;
const themeToggleBtn = document.getElementById('themeToggleBtn');

const icons = {
  light: `<svg xmlns="http://www.w3.org/2000/svg" height="24px" viewBox="0 -960 960 960" width="24px" fill="#e3e3e3"><path d="M480-360q50 0 85-35t35-85q0-50-35-85t-85-35q-50 0-85 35t-35 85q0 50 35 85t85 35Zm0 80q-83 0-141.5-58.5T280-480q0-83 58.5-141.5T480-680q83 0 141.5 58.5T680-480q0 83-58.5 141.5T480-280ZM200-440H40v-80h160v80Zm720 0H760v-80h160v80ZM440-760v-160h80v160h-80Zm0 720v-160h80v160h-80ZM256-650l-101-97 57-59 96 100-52 56Zm492 496-97-101 53-55 101 97-57 59Zm-98-550 97-101 59 57-100 96-56-52ZM154-212l101-97 55 53-97 101-59-57Zm326-268Z"/></svg>`,
  dark: `<svg xmlns="http://www.w3.org/2000/svg" height="24px" viewBox="0 -960 960 960" width="24px" fill="#e3e3e3"><path d="M480-120q-150 0-255-105T120-480q0-150 105-255t255-105q14 0 27.5 1t26.5 3q-41 29-65.5 75.5T444-660q0 90 63 153t153 63q55 0 101-24.5t75-65.5q2 13 3 26.5t1 27.5q0 150-105 255T480-120Zm0-80q88 0 158-48.5T740-375q-20 5-40 8t-40 3q-123 0-209.5-86.5T364-660q0-20 3-40t8-40q-78 32-126.5 102T200-480q0 116 82 198t198 82Zm-10-270Z"/></svg>`,
  system: `<svg xmlns="http://www.w3.org/2000/svg" height="24px" viewBox="0 -960 960 960" width="24px" fill="#e3e3e3"><path d="M480-80q-83 0-156-31.5T197-197q-54-54-85.5-127T80-480q0-83 31.5-156T197-763q54-54 127-85.5T480-880q83 0 156 31.5T763-763q54 54 85.5 127T880-480q0 83-31.5 156T763-197q-54 54-127 85.5T480-80Zm40-83q119-15 199.5-104.5T800-480q0-123-80.5-212.5T520-797v634Z"/></svg>`
};

function getSystemTheme() {
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
}

function setTheme(mode) {
  if (mode === 'system') {
    body.className = 'system';
    themeToggleBtn.innerHTML = icons.system;
  } else {
    body.className = mode;
    themeToggleBtn.innerHTML = icons[mode];
  }

  localStorage.setItem('themeMode', mode);
}

let currentMode = localStorage.getItem('themeMode') || 'system';
setTheme(currentMode);

themeToggleBtn.addEventListener('click', () => {
  if (currentMode === 'system') {
    currentMode = getSystemTheme() === 'dark' ? 'light' : 'dark';
  } else {
    currentMode = 'system';
  }
  setTheme(currentMode);
});

window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
  if (currentMode === 'system') {
    setTheme('system');
  }
});

function process_copy_button_click(id, klass) {
    try {
        const command = document.querySelector(`.command.${klass}`);
        navigator.clipboard.writeText(command.textContent).then(() =>
          document.getElementById(id).style.opacity = '1');

        setTimeout(() => document.getElementById(id).style.opacity = '0', 3000);
    } catch (e) {
        console.log('Hit a snag when copying to clipboard: ', e);
    }
}

function handle_copy_button_click(e) {
    switch (e.id) {
        case 'copy-button-ubuntu':
            process_copy_button_click('copy-status-message-ubuntu', 'ubuntu');
            break;
        case 'copy-button-debian':
            process_copy_button_click('copy-status-message-debian', 'debian');
            break;
        case 'copy-button-fedora':
            process_copy_button_click('copy-status-message-fedora', 'fedora');
            break;
        case 'copy-button-suse':
            process_copy_button_click('copy-status-message-suse', 'suse');
            break;
    }
}

function set_up_copy_button_clicks() {
    var buttons = document.querySelectorAll(".copy-button");
    buttons.forEach(function (element) {
        element.addEventListener('click', function() {
            handle_copy_button_click(element);
        });
    })
}

set_up_copy_button_clicks();

function set_ubuntu(owner, repo) {
    const ubuntu = document.querySelector(".command.ubuntu");

    console.log("Setting Ubuntu install command for:", ubuntu);

    ubuntu.textContent = `wget -qO- https://packhub.dev/sh/ubuntu/github/${owner}/${repo} | sh`
}

function set_debian(owner, repo) {
    const ubuntu = document.querySelector(".command.debian");

    console.log("Setting Ubuntu install command for:", ubuntu);

    ubuntu.textContent = `wget -qO- https://packhub.dev/sh/debian/github/${owner}/${repo} | sh`
}

function set_fedora(owner, repo) {
    const rpm = document.querySelector(".command.fedora");

    console.log("Setting Fedora install command for:", rpm);

    rpm.textContent = `wget -qO- https://packhub.dev/sh/yum/github/${owner}/${repo} | sh`
}

function set_suse(owner, repo) {
    const rpm = document.querySelector(".command.suse");

    console.log("Setting Suse install command for:", rpm);

    rpm.textContent = `wget -qO- https://packhub.dev/sh/zypp/github/${owner}/${repo} | sh`
}

document.addEventListener("DOMContentLoaded", () => {
    const inputElement = document.querySelector(".github-link");

    function extractGithubInfo(value) {
        const githubRegex = /https?:\/\/github\.com\/([^\/]+)\/([^\/]+)/;
        const match = value.match(githubRegex);
        
        if (match) {
            set_ubuntu(match[1], match[2]);
            set_debian(match[1], match[2]);
            set_fedora(match[1], match[2]);
            set_suse(match[1], match[2]);
        } else {
            console.log("Invalid or missing GitHub URL");
        }
    }

    if (inputElement) {
        // Extract initial value (if present)
        extractGithubInfo(inputElement.value || inputElement.placeholder);
        
        // Listen for changes in the input field
        inputElement.addEventListener("input", (event) => {
            extractGithubInfo(inputElement.value || inputElement.placeholder);
        });
    }
});

