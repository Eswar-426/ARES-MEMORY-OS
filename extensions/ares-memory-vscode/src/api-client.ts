import axios from 'axios';

export class ApiClient {
    constructor(private baseUrl: string) {}

    async saveMemory(projectName: string) {
        // Implement save memory API call
        console.log(`Saving memory for ${projectName}`);
    }

    async generateSnapshot(path: string) {
        const response = await axios.post(`${this.baseUrl}/scan`, { path });
        return response.data;
    }

    async getProjectContext(projectId: string) {
        const response = await axios.get(`${this.baseUrl}/project/${projectId}/context`);
        return response.data;
    }

    async searchMemory(query: string) {
        const response = await axios.post(`${this.baseUrl}/memory/search`, { query });
        return response.data;
    }

    async exportContext(projectId: string) {
        const response = await axios.get(`${this.baseUrl}/project/${projectId}/context`);
        return response.data;
    }
}
