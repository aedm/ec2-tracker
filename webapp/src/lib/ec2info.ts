const MACHINE_TYPES = ['medium', 'large', 'xlarge', '2xlarge', '4xlarge', '8xlarge', '10xlarge', '16xlarge', '32xlarge'];

export interface Instance {
    id: string;
    machineClass: string;
    size: number;
    cpuCount: number;
    memory: number;
}

export function getInstance(instanceType: string): string {
    const machineType = instanceType.split('.')[0];
    if (MACHINE_TYPES.includes(machineType)) {
        return machineType;
    }
    return 'unknown';
}