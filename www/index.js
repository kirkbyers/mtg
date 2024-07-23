// Mock data for demonstration
const mockCards = [
    { name: "Lightning Bolt", manaCost: "{R}", type: "Instant", imageUrl: "https://gatherer.wizards.com/Handlers/Image.ashx?multiverseid=191089&type=card" },
    { name: "Black Lotus", manaCost: "{0}", type: "Artifact", imageUrl: "https://gatherer.wizards.com/Handlers/Image.ashx?multiverseid=382866&type=card" },
    { name: "Counterspell", manaCost: "{U}{U}", type: "Instant", imageUrl: "https://gatherer.wizards.com/Handlers/Image.ashx?multiverseid=202437&type=card" },
    { name: "Birds of Paradise", manaCost: "{G}", type: "Creature", imageUrl: "https://gatherer.wizards.com/Handlers/Image.ashx?multiverseid=221896&type=card" },
    { name: "Wrath of God", manaCost: "{2}{W}{W}", type: "Sorcery", imageUrl: "https://gatherer.wizards.com/Handlers/Image.ashx?multiverseid=413580&type=card" },
    { name: "Dark Ritual", manaCost: "{B}", type: "Instant", imageUrl: "https://gatherer.wizards.com/Handlers/Image.ashx?multiverseid=221510&type=card" },
];

const cardGrid = document.getElementById('card-grid');
const searchInput = document.getElementById('search');
const loadMoreButton = document.getElementById('load-more');

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
    const filteredCards = mockCards.filter(card => 
        card.name.toLowerCase().includes(searchTerm) ||
        card.type.toLowerCase().includes(searchTerm)
    );
    renderCards(filteredCards);
  }, 300);
}

searchInput.addEventListener('input', searchCards);
loadMoreButton.addEventListener('click', () => {
    // In a real app, this would load more cards from the server
    alert('In a real app, this would load more cards.');
});

// Initial render
renderCards(mockCards);
