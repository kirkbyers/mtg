let urlParams = new URLSearchParams(window.location.search);
let urlPage = parseInt(urlParams.get('page') || 1);
let urlLimit = parseInt(urlParams.get('limit') || 10);
let urlSearch = urlParams.get('search');

function getCardData(page = 1, limit = 10, search = '') {
  // TODO: Fix this jank. Make args option and check for undef
  page = page === 1 && urlPage ? parseInt(urlPage) : page;
  limit = limit === 10 && urlLimit ? parseInt(urlLimit) : limit;
  search = search === '' && urlSearch ? urlSearch : search;

  const url = `/api/cards?page=${page}&limit=${limit}&search=${search}`;
  return fetch(url)
    .then(response => response.json())
    .then(data => {
      const formattedCards = data.map(card => ({
        name: card.name,
        manaCost: card.mana_cost,
        type: card.type_line,
        imageUrl: card.image_url
      }));
      renderCards(formattedCards);
      // Update the URL with the query parameters
      const queryParams = new URLSearchParams({ page, limit, search }).toString();
      history.pushState(null, null, `?${queryParams}`);
    })
    .catch(error => {
      console.error('Error fetching card data:', error);
    });
}

const cardGrid = document.getElementById('card-grid');
const searchInput = document.getElementById('search');
searchInput.value = urlSearch;
const loadMoreButton = document.getElementById('load-more');

// TODO: infinite scrolling
function renderCards(cards) {
    cardGrid.innerHTML = '';
    cards.forEach(card => {
        const cardElement = document.createElement('div');
        cardElement.className = 'card';
        cardElement.innerHTML = `
            <img src="${card.imageUrl}" alt="${card.name}">
            <h3>${card.name}</h3>
            <p>Mana Cost: ${card.manaCost}</p>
            <p>Type: ${card.type}</p>
        `;
        cardGrid.appendChild(cardElement);
    });
}

let debounceTimeout;

function searchCards() {
  clearTimeout(debounceTimeout);
  debounceTimeout = setTimeout(() => {
    const searchTerm = searchInput.value.toLowerCase();
    getCardData(1, 10, searchTerm);
  }, 300);
}

searchInput.addEventListener('input', searchCards);
loadMoreButton.addEventListener('click', () => {
  const queryParams = new URLSearchParams({ page: urlPage + 1, limit: urlLimit, search: urlSearch }).toString();
  history.pushState(null, null, `?${queryParams}`);
  getCardData(urlPage + 1);
  window.scrollTo(0, 0);
  parseUrlParams();
});

const parseUrlParams = () => {
  urlParams = new URLSearchParams(window.location.search);
  urlPage = parseInt(urlParams.get('page') || 1);
  urlLimit = parseInt(urlParams.get('limit') || 10);
  urlSearch = urlParams.get('search');
}

// Initial render
getCardData();
