#include <iostream>
#include <vector>
#include <chrono>
#include <mpi.h>
#include <algorithm>

void generate(std::vector<int> &v,size_t n) {
    v.reserve(n);
    for(size_t i=0; i<n; ++i) {
        v.push_back((i*13)%100);
    }
    /*v.push_back(1);
    v.push_back(1);
    v.push_back(1);*/
}

// O(n^2)
void workerN2(int id) {
    int n;
    MPI_Status status;
    MPI_Recv(&n,1,MPI_INT,0,1,MPI_COMM_WORLD,&status);

    int dec;
    MPI_Recv(&dec,1,MPI_INT,0,2,MPI_COMM_WORLD,&status);

    std::vector<int> batch;
    batch.resize(n);

    MPI_Recv(batch.data(),n,MPI_INT,0,3,MPI_COMM_WORLD,&status);

    int p1Size;
    MPI_Recv(&p1Size,1,MPI_INT,0,4,MPI_COMM_WORLD,&status);

    std::vector<int> p1;
    p1.resize(p1Size);

    MPI_Recv(p1.data(),p1Size,MPI_INT,0,5,MPI_COMM_WORLD,&status);

    int p2Size;
    MPI_Recv(&p2Size,1,MPI_INT,0,6,MPI_COMM_WORLD,&status);

    std::vector<int> p2;
    p2.resize(p2Size);

    MPI_Recv(p2.data(),p2Size,MPI_INT,0,7,MPI_COMM_WORLD,&status);

    for(int i=0; i<n; i++) {
        int lft=std::max(0, i+dec-(int)p2.size()+1);
        int rgt=std::min((int)p1.size(), i+dec+1);

        for(int j=lft; j<rgt; j++) {
            int other=i+dec-j;
            int add=p1[j]*p2[other];
            batch[i]+=add;
        }
    }

    MPI_Send(batch.data(),n,MPI_INT,0,8,MPI_COMM_WORLD);
}

void masterN2(const std::vector<int> &p1,const std::vector<int> &p2,int nrProc) {
    int resultLength=p1.size()-1+p2.size()-1+1;
    std::vector<int> result;
    result.resize(resultLength);
    for(int i=1; i<nrProc; i++) {
        int lft=((i-1)*resultLength)/(nrProc-1);
        int rgt=(i*resultLength)/(nrProc-1);
        int batchSize=rgt-lft;

        MPI_Send(&batchSize,1,MPI_INT,i,1,MPI_COMM_WORLD);
        MPI_Send(&lft,1,MPI_INT,i,2,MPI_COMM_WORLD);
        MPI_Send(result.data()+lft,batchSize,MPI_INT,i,3,MPI_COMM_WORLD);

        int p1Size=p1.size();
        MPI_Send(&p1Size,1,MPI_INT,i,4,MPI_COMM_WORLD);
        MPI_Send(p1.data(),p1Size,MPI_INT,i,5,MPI_COMM_WORLD);

        int p2Size=p2.size();
        MPI_Send(&p2Size,1,MPI_INT,i,6,MPI_COMM_WORLD);
        MPI_Send(p2.data(),p2Size,MPI_INT,i,7,MPI_COMM_WORLD);
    }

    for(int i=1; i<nrProc; i++) {
        int lft=((i-1)*resultLength)/(nrProc-1);
        int rgt=(i*resultLength)/(nrProc-1);
        int batchSize=rgt-lft;

        MPI_Status status;
        MPI_Recv(result.data()+lft,batchSize,MPI_INT,i,8,MPI_COMM_WORLD,&status);
    }

    /*for(int i=0; i<resultLength; i++) {
        std::cout<<result[i]<<" ";
    }
    std::cout<<"\n";*/
}

// Karatsuba
void add(int *a,int sza,int *b,int szb,std::vector<int> &res) {
    int sz=std::max(sza,szb);
    res.resize(sz);
    for(int i=0; i<sz; i++) {
        int valA=0;
        int valB=0;

        if(i<sza) {
            valA=a[i];
        }
        if(i<szb) {
            valB=b[i];
        }

        res[i]=valA+valB;
    }
}

void sub(int *a,int sza,int *b,int szb,int *c,int szc,std::vector<int> &res) {
    int sz=std::max(sza,std::max(szb,szc));
    res.resize(sz);
    for(int i=0; i<sz; i++) {
        int valA=0;
        int valB=0;
        int valC=0;

        if(i<sza) {
            valA=a[i];
        }
        if(i<szb) {
            valB=b[i];
        }
        if(i<szc) {
            valC=c[i];
        }

        res[i]=valA-valB-valC;
    }
}

void karatsuba(int* p1,int np1,int* p2,int np2,int *res,int me,int nrProcs) {
    if(np1<=16||np2<=16) {
        for(int i=0; i<np1; i++) {
            for(int j=0; j<np2; j++) {
                res[i+j]=p1[i]*p2[j];
            }
        }
        return;
    }
    int n=std::max(np1,np2)/2;

    int np1Low=std::min(np1,n);
    int np1High=np1-np1Low;
    int np2Low=std::min(np2,n);
    int np2High=np2-np2Low;

    std::vector<int> a,d,e;
    a.resize(np1High-1+np2High-1+1);
    d.resize(np1Low-1+np2Low-1+1);

    if(nrProcs>=3) {
        int child1=me+nrProcs/3;
        int child2=me+(2*nrProcs)/3;

        // a = x_high * y_high
        // d = x_low * y_low
        // e = (x_high + x_low) * (y_high + y_low) - a - d

        //sizes[1]=nrProcs-nrProcs/2;
        //cout<<"Worker "<<me<<", sending to child "<<child<<", part size = "<<n-k<<", nrProcs = "<<sizes[1]<<"\n";

        // x_high * y_high
        int highProcs=(2*nrProcs)/3-nrProcs/3;
        MPI_Send(&highProcs,1,MPI_INT,child1,1,MPI_COMM_WORLD);
        MPI_Send(&np1High,1,MPI_INT,child1,2,MPI_COMM_WORLD);
        MPI_Send(p1+np1Low,np1High,MPI_INT,child1,3,MPI_COMM_WORLD);

        MPI_Send(&np2High,1,MPI_INT,child1,4,MPI_COMM_WORLD);
        MPI_Send(p2+np2Low,np2High,MPI_INT,child1,5,MPI_COMM_WORLD);
        
        // x_low * y_low
        int lowProcs=nrProcs-(2*nrProcs)/3;
        MPI_Send(&lowProcs,1,MPI_INT,child2,1,MPI_COMM_WORLD);
        MPI_Send(&np1Low,1,MPI_INT,child2,2,MPI_COMM_WORLD);
        MPI_Send(p1,np1Low,MPI_INT,child2,3,MPI_COMM_WORLD);

        MPI_Send(&np2Low,1,MPI_INT,child2,4,MPI_COMM_WORLD);
        MPI_Send(p2,np2Low,MPI_INT,child2,5,MPI_COMM_WORLD);

        // (x_high + x_low) * (y_high + y_low)
        std::vector<int> eWoSub;
        int eWoSubSize=std::max(np1Low,np1High)-1+std::max(np2Low,np2High)-1+1;
        eWoSub.resize(eWoSubSize);

        std::vector<int> x,y;
        add(p1,np1Low,p1+np1Low,np1High,x);
        add(p2,np2Low,p2+np2Low,np2High,y);

        karatsuba(x.data(),x.size(),y.data(),y.size(),eWoSub.data(),me,nrProcs/3);
        
        MPI_Status status;
        MPI_Recv(a.data(),a.size(),MPI_INT,child1,6,MPI_COMM_WORLD,&status);
        MPI_Recv(d.data(),d.size(),MPI_INT,child2,6,MPI_COMM_WORLD,&status);

        sub(eWoSub.data(),eWoSub.size(),a.data(),a.size(),d.data(),d.size(),e);
    }
    else {
        if(nrProcs==2) {
            int val=-1;
            MPI_Send(&val,1,MPI_INT,me+1,1,MPI_COMM_WORLD);
        }
        // x_low * y_low
        karatsuba(p1,np1Low,p2,np2Low,d.data(),me,1);

        // x_high * y_high
        karatsuba(p1+np1Low,np1High,p2+np2Low,np2High,a.data(),me,1);

        // (x_high + x_low) * (y_high + y_low)
        std::vector<int> eWoSub;
        int eWoSubSize=std::max(np1Low,np1High)-1+std::max(np2Low,np2High)-1+1;
        eWoSub.resize(eWoSubSize);

        std::vector<int> x,y;
        add(p1,np1Low,p1+np1Low,np1High,x);
        add(p2,np2Low,p2+np2Low,np2High,y);

        karatsuba(x.data(),x.size(),y.data(),y.size(),eWoSub.data(),me,1);

        sub(eWoSub.data(),eWoSub.size(),a.data(),a.size(),d.data(),d.size(),e);
    }
    
    for(int i=0; i<a.size(); i++) {
        res[i+2*n]+=a[i];
    }

    for(int i=0; i<d.size(); i++) {
        res[i]+=d[i];
    }

    for(int i=0; i<e.size(); i++) {
        res[i+n]+=e[i];
    }
}

void root(std::vector<int> &p1,std::vector<int> &p2,int nrProcs) {
    std::vector<int> res;
    res.resize(p1.size()-1+p2.size()-1+1);
    karatsuba(p1.data(),p1.size(),p2.data(),p2.size(),res.data(),0,nrProcs);

    /*for(int i=0; i<p1.size()-1+p2.size()-1+1; i++) {
        std::cout<<res[i]<<" ";
    }
    std::cout<<"\n";*/
}

void workerKaratsuba(int me) {
    MPI_Status status;
    int procs;
    MPI_Recv(&procs,1,MPI_INT,MPI_ANY_SOURCE,1,MPI_COMM_WORLD,&status);
    int parent=status.MPI_SOURCE;

    if(procs==-1) {
        return;
    }

    int np1;
    MPI_Recv(&np1,1,MPI_INT,parent,2,MPI_COMM_WORLD,&status);

    std::cout<<"Worker "<<me<<", child of "<<parent<<", part size = "<<np1<<", nrProcs = "<<procs<<std::endl;

    std::vector<int> p1;
    p1.resize(np1);
    MPI_Recv(p1.data(),np1,MPI_INT,parent,3,MPI_COMM_WORLD,&status);

    int np2;
    MPI_Recv(&np2,1,MPI_INT,parent,4,MPI_COMM_WORLD,&status);

    std::vector<int> p2;
    p2.resize(np2);
    MPI_Recv(p2.data(),np2,MPI_INT,parent,5,MPI_COMM_WORLD,&status);

    std::vector<int> res;
    if(np1!=0 && np2!=0) {
        res.resize(np1-1+np2-1+1);

        karatsuba(p1.data(),p1.size(),p2.data(),p2.size(),res.data(),me,procs);
    }

    // send back the result to the parent
    //cout<<"Worker "<<me<<", sending to parent "<<parent<<", part size = "<<n<<"\n";
    MPI_Ssend(res.data(),res.size(),MPI_INT,parent,6,MPI_COMM_WORLD);
}

int main() {
    MPI_Init(0,0);
    int me;
    int nrProcs;
    MPI_Comm_size(MPI_COMM_WORLD,&nrProcs);
    MPI_Comm_rank(MPI_COMM_WORLD,&me);
    //std::cout<<"Hello, I am "<<me<<" out of "<<nrProcs<<"\n";

    if(me==0) {
        std::vector<int> p1;
        std::vector<int> p2;
        generate(p1,10000);
        generate(p2,10000);

        std::chrono::high_resolution_clock::time_point const beginTime=std::chrono::high_resolution_clock::now();

        //masterN2(p1, p2, nrProcs);
        root(p1,p2,nrProcs);

        std::chrono::high_resolution_clock::time_point const endTime=std::chrono::high_resolution_clock::now();

        printf("Fished, time=%lldms\n",
            (std::chrono::duration_cast<std::chrono::milliseconds>(endTime-beginTime)).count());
    }
    else {
        // worker
        std::chrono::high_resolution_clock::time_point const beginTime=std::chrono::high_resolution_clock::now();
        //workerN2(me);
        workerKaratsuba(me);
        std::chrono::high_resolution_clock::time_point const endTime=std::chrono::high_resolution_clock::now();

        printf("(worker %d): time=%lldms\n",me,
            (std::chrono::duration_cast<std::chrono::milliseconds>(endTime-beginTime)).count());
    }
    MPI_Finalize();
}