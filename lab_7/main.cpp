#include <iostream>
#include <vector>
#include <chrono>
#include <mpi.h>

void generate(std::vector<int>& v, size_t n) {
    std::cout << v.size();
}

// O(n^2)
void workerN2(int id) {
    int n;
    MPI_Status status;
    MPI_Recv(&n, 1, MPI_INT, 0, 1, MPI_COMM_WORLD, &status);

    std::vector<int> batch;
    batch.resize(n);

    MPI_Recv(batch.data(), n, MPI_INT, 0, 2, MPI_COMM_WORLD, &status);

    batch[0] = id;

    MPI_Send(batch.data(), n, MPI_INT, 0, 3, MPI_COMM_WORLD);
}

void masterN2(const std::vector<int> &p1, const std::vector<int> &p2, int nrProc) {
    int resultLength = p1.size() - 1 + p2.size() - 1 + 1;
    std::vector<int> result;
    result.resize(resultLength);
    for (int i = 1; i < nrProc; i++) {
        int lft = ((i - 1) * resultLength) / (nrProc - 1);
        int rgt = (i * resultLength) / (nrProc - 1);
        int batchSize = rgt - lft;

        MPI_Send(&batchSize, 1, MPI_INT, i, 1, MPI_COMM_WORLD);
        MPI_Send(result.data() + lft, batchSize, MPI_INT, i, 2, MPI_COMM_WORLD);
    }

    for (int i = 1; i < nrProc; i++) {
        int lft = ((i - 1) * resultLength) / (nrProc - 1);
        int rgt = (i * resultLength) / (nrProc - 1);
        int batchSize = rgt - lft;

        MPI_Status status;
        MPI_Recv(result.data() + lft, batchSize, MPI_INT, i, 3, MPI_COMM_WORLD, &status);
    }

    for (int i = 0; i < resultLength; i++) {
        std::cout << result[i] << " ";
    }
}

int main() {
    MPI_Init(0, 0);
    int me;
    int nrProcs;
    MPI_Comm_size(MPI_COMM_WORLD, &nrProcs);
    MPI_Comm_rank(MPI_COMM_WORLD, &me);
    std::cout << "Hello, I am " << me << " out of " << nrProcs << "\n";

    if(me == 0) {
        std::chrono::high_resolution_clock::time_point const beginTime = std::chrono::high_resolution_clock::now();

        std::vector<int> p1;
        std::vector<int> p2;
        generate(p1, 3);
        generate(p2, 3);

        //masterN2(p1, p2, nrProcs);

        std::chrono::high_resolution_clock::time_point const endTime = std::chrono::high_resolution_clock::now();

        printf("Fished, time=%lldms\n",
               (std::chrono::duration_cast<std::chrono::milliseconds>(endTime-beginTime)).count());
    } else {
        // worker
        std::chrono::high_resolution_clock::time_point const beginTime = std::chrono::high_resolution_clock::now();
        //workerN2(me);
        std::chrono::high_resolution_clock::time_point const endTime = std::chrono::high_resolution_clock::now();

        printf("(worker %d): time=%lldms\n", me,
               (std::chrono::duration_cast<std::chrono::milliseconds>(endTime-beginTime)).count());
    }
    MPI_Finalize();
}