import axios from 'axios';

export const checkNetworkStatus = async () => {
    try {
        const response = await axios.get('https://cdn.bootcdn.net/ajax/libs/lodash.js/4.17.21/lodash.core.min.js');
        return response.status === 200;
    } catch {
        return false;
    }

};

